Structured Data Communication with PTY-Based LLM Agents in Rust

Establishing a reliable protocol between a Rust orchestrator and sandboxed CLI LLM agents requires careful handling of the PTY stream. The goal is to parse infrequent structured messages (JSON wrapped in <FORGE_EVENT> tags) from a continuous stream of mostly normal text. Below we outline best practices and potential pitfalls, and provide a robust Rust parsing pattern.

Protocol Design and Unique Delimiters
	•	Use Unique Delimiter Tags: Wrap structured JSON messages in a distinctive tag format (e.g. <FORGE_EVENT name="..."> ... </FORGE_EVENT>). This tag should be unique enough that it never appears in normal agent output ￼. Using a clearly unique keyword like FORGE_EVENT (all-caps, unlikely in casual text) helps distinguish protocol messages from conversational text or code.
	•	Isolation of Structured Data: Ensure the agent only emits these tags for intended events (e.g. PLAN_COMPLETE, TASK_COMPLETE). The vast majority of output remains free-form. By design, structured messages are out-of-band signals that do not interfere with the agent’s normal operation. For example, you might instruct the LLM (via system prompts or the agent’s logic) to only use <FORGE_EVENT> tags for signalling events, and never as part of normal dialogue or code.
	•	Non-Interference with Agent Output: Because the tags are unique and infrequent, they won’t collide with typical text. This ensures the protocol remains invisible to the agent’s primary function. The orchestrator can strip or handle these tags separately, so they aren’t fed back into the LLM or shown to end-users. In practice, the agent can “think out loud” or produce normal text freely; the orchestrator simply ignores those parts when looking for structured events ￼.

Reading from a PTY without Blocking

Reading from a pseudoterminal (PTY) in Rust must be done without blocking indefinitely, since the agent process may run for a long time. Do not use high-level methods like read_to_string() which wait for EOF (end-of-stream) – that will block your thread because a PTY won’t close until the process ends ￼. Instead, use a loop that reads in chunks or use an asynchronous stream:
	•	Continuous Non-Blocking Reads: Use either a dedicated thread or an async task to continuously read from the PTY’s master file descriptor. For example, with the pty_process crate (which provides PTY support), you can spawn the agent with a PTY and then call pty.read() in a loop. Reading in moderate-sized chunks (e.g. 1KB or 4KB at a time) is typical ￼. This avoids blocking the main thread and lets you process output incrementally.
	•	Buffered Reading for Streams: Since output arrives as a byte stream, consider buffering the data manually rather than line-by-line. Reading line-by-line (BufRead::lines()) can block if a line isn’t terminated, and lines might split JSON in the middle. Instead, treat the stream as raw bytes or UTF-8 text and implement your own delimiter scanning ￼. This is effectively a simple parser for the <FORGE_EVENT> markers.

Handling Partial Output and Buffering

Partial reads are expected: The structured <FORGE_EVENT> message might not arrive in one chunk. For example, the agent could output <FORGE_EVENT name="TASK_COMPLETE">{"result": "ok", "details": ...} in one chunk and the closing </FORGE_EVENT> in the next. Your parser must handle this by buffering data until a complete message is received.
	•	Maintain a Rolling Buffer: Keep an in-memory buffer (e.g. a String or Vec<u8>) for data read from the PTY. After each read, append the new chunk to the buffer and then search the buffer for the delimiter patterns. Using a byte buffer can be efficient – you can search for the substring "</FORGE_EVENT>" in it. If you find a start tag <FORGE_EVENT but not the closing tag yet, simply wait for more data. This approach is acceptable since structured messages are infrequent and we don’t require real-time parsing mid-message. Buffering until the closing delimiter is found is explicitly fine for this workflow (no need to parse events until they are complete).
	•	Delimiter Detection: Implement a search for the unique delimiters. A straightforward method is using Vec<u8>::windows() or byte string searches to find the index of the <FORGE_EVENT start and the </FORGE_EVENT> end in the buffer. Ensure that the end tag you find comes after the start tag. (If multiple events could be in the buffer, you might loop to extract all complete <FORGE_EVENT>...</FORGE_EVENT> blocks in one go.)
	•	State Machine (optional): For more complex scenarios, you could implement a small state machine to track whether you’re currently inside a <FORGE_EVENT> block. However, given the infrequent and self-contained nature of these events, a simple search-and-extract on the buffer is often sufficient. The key is to not assume the message is fully in one read – always handle the case where the tags are split across reads ￼.
	•	Avoid Line-Oriented Parsing: As a best practice, do not assume the structured message aligns with newline boundaries. Use raw byte or char scanning. (Lines in JSON or XML can be arbitrarily long, so a line-based read could break in the middle of a JSON object ￼.) Instead, detect the exact delimiter sequence in the stream.
	•	Buffer Cleanup: After you’ve successfully extracted a complete <FORGE_EVENT>...JSON...</FORGE_EVENT> block from the buffer, remove that portion from the buffer to prevent re-processing and to keep memory usage in check. Keep any trailing partial data (if the message ended in the middle of a chunk, etc.). It’s wise to also impose some reasonable size limit on the buffer (in case the agent outputs huge amounts of text without any events, you might periodically truncate or clear old processed data).

Parsing and Validating the JSON Payload

Once a complete structured message is extracted between the tags, the orchestrator should parse and validate it:
	•	Extract the JSON Safely: Typically, the format is <FORGE_EVENT name="XYZ"> { ...JSON... } </FORGE_EVENT>. You can find the position of the first > after the start tag to get the beginning of the JSON content, and the position of the closing </FORGE_EVENT> tag to get the end of the JSON string. Be careful to slice at the correct boundaries (exclude the tags themselves). For example: find index of '> in <FORGE_EVENT name="XYZ"> to start after that, and the index of </FORGE_EVENT> to end before that.
	•	Use a Robust JSON Parser: Utilize a reliable JSON library like Serde (serde_json) to parse the extracted JSON text into a Value or a predefined struct. This is where the Rust orchestrator acts as the source of truth – do not trust the LLM’s output blindly. If serde_json::from_str fails (due to malformed JSON), handle that error gracefully. The orchestrator might log an error or decide to ignore the event if it’s not well-formed. Never assume the LLM will always produce perfect JSON.
	•	Schema Validation: After parsing JSON, validate that it matches the expected schema for the event type. For example, if name="TASK_COMPLETE", you might expect certain keys like "result" or "status" in the JSON. You can enforce this by deserializing into a Rust struct (which fails if required fields are missing), or by using a JSON schema validator (there are crates like jsonschema for runtime validation). This extra step ensures the data conforms to what your orchestrator logic expects ￼. The importance of schema validation is echoed in LLM tooling: even if the JSON format is instructed, the model might produce slight deviations, so a strict check on structure is recommended ￼.
	•	Handling Malformed Data: Potential failure modes include the LLM outputting invalid JSON (e.g. missing a quote or brace), or not closing the tag properly. In these cases, the orchestrator should not crash or hang indefinitely. Implement timeouts or size limits for an event in progress – for instance, if you see a start tag but 10KB of data with no closing tag, you might decide to abandon that parse and reset the buffer or restart the agent. Likewise, if JSON parsing fails, you could attempt a simple recovery (e.g. if a closing brace is missing at EOF, maybe append one and try again) or just log and ignore. The system should be robust to garbage output from the agent.
	•	False Positives: Ensure that the parser doesn’t mistake normal output for an event. With a unique delimiter like <FORGE_EVENT>, this risk is low. Still, design the extraction to only treat something as a valid event if it exactly matches the pattern (including a proper closing tag). If the agent somehow printed a string like "<FORGE_EVENT" in regular text (unlikely if your delimiter is unique and especially if it’s part of a well-formed XML-like tag), the parser might get into a “waiting for end tag” state incorrectly. Mitigate this by choosing a delimiter that the agent is highly unlikely to produce arbitrarily (e.g. a token that would never appear in code or conversation). The uniqueness of the <FORGE_EVENT> tag is crucial for this reason ￼.

Code Pattern: Robust PTY Stream Parser in Rust

Below is an illustrative code snippet demonstrating how you might implement the reading and parsing loop. This uses the pty_process crate for PTY support (for simplicity, error handling is minimal here):

use pty_process::Command;
use std::io::{Read, Result};
use serde_json::Value;

let (mut pty, pts) = pty_process::open()?;            // Open a new PTY pair
pty.resize(pty_process::Size::new(24, 80))?;          // Set PTY size (optional)
let mut child_cmd = Command::new("claude-cli");       // e.g., spawn the Claude CLI
// ... configure arguments, environment, etc.
let mut child = child_cmd.spawn(pts)?;                // Spawn the LLM process attached to PTY

let start_tag = b"<FORGE_EVENT";
let end_tag   = b"</FORGE_EVENT>";
let mut buffer: Vec<u8> = Vec::new();

loop {
    let mut chunk = [0u8; 1024];
    let n = match pty.read(&mut chunk) {             // Read from PTY (non-blocking)
        Ok(0) => break,                              // Child process ended (EOF)
        Ok(n) => n,
        Err(e) => {
            eprintln!("Read error: {}", e);
            break;
        }
    };
    buffer.extend_from_slice(&chunk[..n]);            // Append new data to buffer

    // Search for a complete <FORGE_EVENT> ... </FORGE_EVENT> block in the buffer
    if let Some(start_idx) = buffer.windows(start_tag.len()).position(|w| w == start_tag) {
        if let Some(end_idx_rel) = buffer[start_idx..].windows(end_tag.len()).position(|w| w == end_tag) {
            let end_idx = start_idx + end_idx_rel;
            let event_bytes = &buffer[start_idx .. end_idx + end_tag.len()];
            if let Ok(event_str) = std::str::from_utf8(event_bytes) {
                // Extract JSON substring between '>' and '</FORGE_EVENT>'
                if let Some(json_start) = event_str.find('>') {
                    let json_start_idx = json_start + 1;
                    let json_end_idx = event_str.rfind('<').unwrap_or(event_str.len());
                    let json_text = &event_str[json_start_idx .. json_end_idx];
                    // Attempt to parse JSON
                    match serde_json::from_str::<Value>(json_text) {
                        Ok(json_value) => {
                            let event_name = event_str[ start_tag.len() .. json_start ].trim();
                            println!("Received event {} with data: {}", event_name, json_value);
                            // TODO: validate schema and handle the event
                        }
                        Err(e) => {
                            eprintln!("Malformed JSON in FORGE_EVENT: {}", e);
                        }
                    }
                }
            }
            // Remove the processed event from the buffer to avoid infinite growth
            buffer.drain(.. end_idx + end_tag.len());
        }
    }

    // (Optional) Prevent buffer from growing unbounded if no events are found
    if buffer.len() > 1_000_000 {
        buffer.clear();  // or trim older data safely
    }
}
# child.wait()?;  // ensure the child process is waited on (if needed)

How this works: The loop continuously reads from the PTY master (pty) without waiting for an EOF (this ensures we capture output as it comes ￼). Data is appended to a buffer. We then search the buffer for our start and end tag sequences. Once a complete event is found (both tags present), we extract the JSON substring inside and parse it with Serde. We then remove the processed portion from the buffer, allowing the buffer to slide forward and not grow indefinitely. The search is done on the raw byte buffer for reliability (avoiding any Unicode boundary issues or line breaks issues).

Robustness considerations: We take care not to assume the JSON is valid – any parsing error is caught and logged. In a production setting, you might take additional actions on parse failure (e.g. notify that the agent’s output was malformed). The parser also assumes at most one event in the buffer at a time; if multiple events arrived in one chunk or back-to-back, the loop would catch the first, then continue and catch the next in a subsequent iteration (after removing the first). This approach can be expanded to find all events in the buffer in one go if needed, but infrequent events means a simple one-at-a-time approach is sufficient.

Potential Failure Modes and Mitigations
	•	Partial or Split Tags: If a tag is split across two reads (e.g. one chunk ends with "<FORGE_E" and the next begins with "VENT>"), the buffer concatenation ensures the full sequence <FORGE_EVENT> is eventually in the buffer. The windowed search will find it once it’s complete. Until then, no false trigger occurs because the full delimiter wasn’t present. This is a reason we buffer and search in the combined data, rather than reacting per-chunk.
	•	Missing Closing Tag: If an agent fails to emit the closing </FORGE_EVENT> (due to a bug or crash), the parser would keep buffering data indefinitely waiting for it. To handle this, consider implementing a timeout or length check for an open event. For example, if you see a start tag and yet after X seconds or N kilobytes of data there is still no closing tag, you might abandon the attempt (clear the buffer or reset the state). This prevents the orchestrator from hanging forever on a malformed output.
	•	Malformed JSON Content: As noted, always wrap JSON parsing in error handling. The orchestrator should treat invalid JSON as a non-fatal error of the agent. A best practice is to have the orchestrator either ignore the bad event (and possibly signal the agent to retry or proceed) or attempt a corrective measure (like stripping non-JSON chatter around the braces, if any). The structured format being in a unique tag helps here: you can be fairly confident that everything between the tags was intended as JSON, so if parsing fails, the agent indeed produced bad data. Logging and metrics on such failures can help improve prompts or agent logic over time.
	•	Normal Output Collision: If by some rare chance the agent’s normal text includes the exact delimiter pattern (e.g. it prints a dummy string "<FORGE_EVENT foo>"), the parser might incorrectly attempt to interpret it as an event. To avoid this, choose delimiters that the agent would not naturally produce. In addition, you can add simple sanity checks: for instance, your structured events are expected to contain valid JSON immediately after the > of the start tag. If the parser finds <FORGE_EVENT but what follows isn’t a well-formed JSON object by the time the closing tag appears, you can decide to ignore it as a false positive. This way, even if the sequence occurs in normal output, it likely wouldn’t accidentally form a valid-looking JSON block, so it would fail validation and be skipped.
	•	Multiple or Nested Events: The protocol should ideally avoid nested <FORGE_EVENT> tags. If the agent somehow printed a second <FORGE_EVENT> before closing the first, your parser might find the first start and the second’s close, causing confusion. Defining the protocol to disallow nesting (one event must fully finish before another starts) is important. Your parsing logic can also handle this by always taking the first <FORGE_EVENT> and the first subsequent </FORGE_EVENT> as a pair – any extra <FORGE_EVENT> before the first close tag would either be ignored or treated as part of the JSON text until the first close tag. In summary, do not allow overlapping events in the design, and if they occur due to a bug, treat it as malformed output.

Ensuring Smooth Agent Operation

Finally, ensure that integrating this protocol doesn’t disrupt the agent processes:
	•	Agent Output Behavior: Running the LLM in a PTY can sometimes introduce ANSI color codes or interactive prompts. Make sure to disable or filter out any terminal control sequences from the output if they could appear, since they might appear in the byte stream. These generally won’t include your delimiter patterns, but for cleanliness you might strip them from text before feeding it to the JSON parser (or configure the agent to run in a “dumb” terminal mode with no colors).
	•	Flushing and Newlines: If the agent’s structured messages might be buffered, ensure the agent flushes stdout after printing the event JSON and closing tag. For example, if using Python in the agent, one might do print(event_json, flush=True). In Rust, writing to stdout should be followed by a .flush() call. This ensures the orchestrator doesn’t sit waiting because the data is stuck in the agent’s output buffer. In a PTY, line-buffering might flush on newline automatically, but it’s good to be explicit.
	•	Transparency: The orchestrator can still forward the agent’s normal output (excluding the structured event lines) to logs or UI if needed. The protocol lines could be filtered out or logged separately for debugging. This way, from a user’s perspective, the agent behaves normally, and the structured communication is an invisible back-channel.

Conclusion

Summary of best practices: Use a clearly defined delimiter-based mini-protocol to embed JSON in the agent’s output, making it easy for the Rust orchestrator to detect. Read from the PTY using non-blocking, incremental reads (e.g. via an async task or a thread) ￼, and implement a simple parser that buffers data and searches for the <FORGE_EVENT> ... </FORGE_EVENT> pattern ￼ ￼. Once a complete message is captured, parse the JSON and validate it against your expected schema before trusting it ￼. Prepare for failure modes like incomplete or malformed output by timing out or error-checking, rather than assuming perfection. By following these practices, you can achieve robust, structured communication with LLM agent processes without disrupting their normal conversational output. The result is a reliable orchestration layer that knows how to separate structured signals from free-form text and handle each appropriately.

Sources:
	•	Rust forum discussion on non-blocking PTY reads ￼
	•	Advice on implementing custom stream parsers with delimiters ￼ ￼
	•	Techniques for parsing and validating LLM outputs (JSON schemas and robust parsing) ￼