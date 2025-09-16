# Structured Data Communication with PTY-Based LLM Agents in Rust

Establishing a reliable protocol between a Rust orchestrator and
sandboxed CLI LLM agents requires careful handling of the PTY stream.
The goal is to parse infrequent **structured messages** (JSON wrapped in
`<FORGE_EVENT>` tags) from a continuous stream of mostly normal text.
Below we outline best practices and potential pitfalls, and provide a
robust Rust parsing pattern.

## Protocol Design and Unique Delimiters

-   **Use Unique Delimiter Tags:** Wrap structured JSON messages in a
    distinctive tag format
    (e.g. `<FORGE_EVENT name="..."> ... </FORGE_EVENT>`). This tag
    should be unique enough that it **never appears in normal agent
    output**. Using a clearly unique keyword like `FORGE_EVENT`
    (all-caps, unlikely in casual text) helps distinguish protocol
    messages from conversational text or code.\
-   **Isolation of Structured Data:** Ensure the agent only emits these
    tags for intended events (e.g. `PLAN_COMPLETE`, `TASK_COMPLETE`).
    The vast majority of output remains free-form. By design, structured
    messages are **out-of-band** signals that do not interfere with the
    agent's normal operation. For example, you might instruct the LLM
    (via system prompts or the agent's logic) to only use
    `<FORGE_EVENT>` tags for signalling events, and never as part of
    normal dialogue or code.\
-   **Non-Interference with Agent Output:** Because the tags are unique
    and infrequent, they won't collide with typical text. This ensures
    the protocol remains invisible to the agent's primary function. The
    orchestrator can strip or handle these tags separately, so they
    aren't fed back into the LLM or shown to end-users. In practice, the
    agent can "think out loud" or produce normal text freely; the
    orchestrator simply ignores those parts when looking for structured
    events.

## Reading from a PTY without Blocking

Reading from a pseudoterminal (PTY) in Rust must be done without
blocking indefinitely, since the agent process may run for a long time.
**Do not use** high-level methods like `read_to_string()` which wait for
EOF (end-of-stream) -- that will block your thread because a PTY **won't
close until the process ends**. Instead, use a loop that reads in chunks
or use an asynchronous stream:

-   **Continuous Non-Blocking Reads:** Use either a dedicated thread or
    an async task to continuously read from the PTY's master file
    descriptor. For example, with the
    [**`pty_process`**](https://docs.rs/pty-process) crate (which
    provides PTY support), you can spawn the agent with a PTY and then
    call `pty.read()` in a loop. Reading in moderate-sized chunks
    (e.g. 1KB or 4KB at a time) is typical. This avoids blocking the
    main thread and lets you process output incrementally.\
-   **Buffered Reading for Streams:** Since output arrives as a byte
    stream, consider buffering the data manually rather than
    line-by-line. Reading line-by-line (`BufRead::lines()`) can block if
    a line isn't terminated, and lines might split JSON in the middle.
    Instead, treat the stream as raw bytes or UTF-8 text and implement
    your own delimiter scanning. This is effectively a simple parser for
    the `<FORGE_EVENT>` markers.

## Handling Partial Output and Buffering

**Partial reads are expected:** The structured `<FORGE_EVENT>` message
might not arrive in one chunk. For example, the agent could output
`<FORGE_EVENT name="TASK_COMPLETE">{"result": "ok", "details": ...}` in
one chunk and the closing `</FORGE_EVENT>` in the next. Your parser must
handle this by **buffering data until a complete message is received**.

-   **Maintain a Rolling Buffer:** Keep an in-memory buffer (e.g. a
    `String` or `Vec<u8>`) for data read from the PTY. After each read,
    append the new chunk to the buffer and then search the buffer for
    the delimiter patterns. Using a byte buffer can be efficient -- you
    can search for the substring `"</FORGE_EVENT>"` in it. If you find a
    start tag `<FORGE_EVENT` but not the closing tag yet, simply wait
    for more data.\
-   **Delimiter Detection:** Implement a search for the unique
    delimiters. A straightforward method is using `Vec<u8>::windows()`
    or byte string searches to find the index of the `<FORGE_EVENT`
    start and the `</FORGE_EVENT>` end in the buffer. Ensure that the
    end tag you find comes *after* the start tag.\
-   **Buffer Cleanup:** After you've successfully extracted a complete
    `<FORGE_EVENT>...JSON...</FORGE_EVENT>` block from the buffer,
    remove that portion from the buffer to prevent re-processing and to
    keep memory usage in check.

## Parsing and Validating the JSON Payload

-   **Extract the JSON Safely:** Find the position of the first `>`
    after the start tag to get the beginning of the JSON content, and
    the position of the closing `</FORGE_EVENT>` tag to get the end of
    the JSON string.\
-   **Use a Robust JSON Parser:** Use Serde (`serde_json`) to parse the
    extracted JSON text. If parsing fails, handle gracefully (log or
    ignore the event).\
-   **Schema Validation:** Validate the JSON structure against an
    expected schema using either Rust structs or a schema validator
    crate.

## Rust Code Example

``` rust
use pty_process::Command;
use std::io::Read;
use serde_json::Value;

let (mut pty, pts) = pty_process::open()?;
let mut child = Command::new("claude-cli").spawn(pts)?;

let start_tag = b"<FORGE_EVENT";
let end_tag   = b"</FORGE_EVENT>";
let mut buffer: Vec<u8> = Vec::new();

loop {
    let mut chunk = [0u8; 1024];
    let n = match pty.read(&mut chunk) {
        Ok(0) => break,
        Ok(n) => n,
        Err(_) => break,
    };
    buffer.extend_from_slice(&chunk[..n]);

    if let Some(start_idx) = buffer.windows(start_tag.len()).position(|w| w == start_tag) {
        if let Some(end_idx_rel) = buffer[start_idx..].windows(end_tag.len()).position(|w| w == end_tag) {
            let end_idx = start_idx + end_idx_rel;
            let event_bytes = &buffer[start_idx .. end_idx + end_tag.len()];
            if let Ok(event_str) = std::str::from_utf8(event_bytes) {
                if let Some(json_start) = event_str.find('>') {
                    let json_start_idx = json_start + 1;
                    let json_end_idx = event_str.rfind('<').unwrap_or(event_str.len());
                    let json_text = &event_str[json_start_idx .. json_end_idx];
                    if let Ok(json_value) = serde_json::from_str::<Value>(json_text) {
                        println!("Received event: {}", json_value);
                    }
                }
            }
            buffer.drain(.. end_idx + end_tag.len());
        }
    }
}
```

## Failure Modes

-   **Split tags:** Buffer ensures detection once complete.\
-   **Missing closing tag:** Use timeouts or size limits to abandon
    incomplete events.\
-   **Malformed JSON:** Handle parse errors gracefully.\
-   **Nested/overlapping tags:** Disallow nesting in protocol design.\
-   **Collision with normal output:** Mitigated by using unique
    delimiters.
