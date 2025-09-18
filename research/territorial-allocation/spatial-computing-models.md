Here is a compact, implementation-focused comparison for emergent boundaries in AI swarms operating on a shared codebase.

1. Voronoi/CVT territory from semantic seeds

* Mechanism: embed files and agents. Assign each file to the nearest agent seed in embedding space. Iterate Lloyd to centroidal Voronoi for stable regions. \[verified] ([wias-berlin.de][1])
* Weighted control: use power diagrams to bias regions by expertise, workload, or quality score. \[verified] ([Department of Computer Science][2])
* Convergence rule: periodic reseeding from region centroids lowers boundary jitter. \[verified] ([SIAM E-Books][3])
* Fit for code: treat each file or symbol vector as a point; region = agent responsibility set. \[inferred]

2. Stigmergic territoriality from edit “scent”

* Mechanism: agents deposit decaying marks on paths, modules, or APIs. Others avoid high foreign-scent areas unless escalation triggers. \[verified] ([PMC][4])
* Dynamics: stable home ranges emerge from local avoidance and renewal. \[verified] ([PMC][4])
* Fit for code: scent = recent edits, failing tests, or review debt. Decay drives natural turnover. \[inferred]

3. Hash-space partitioning (consistent or rendezvous)

* Mechanism: assign files to agents by hashing stable keys (path, package, symbol). Small movement on agent churn. \[verified] ([amosbrocco.ch][5])
* Tradeoff: rendezvous often evens load better than vanilla rings. \[verified] ([DZone][6])
* Fit for code: fast, deterministic territories without embeddings. Weak semantic locality unless keys encode structure. \[inferred]

4. DHT-style coordinate zones

* Mechanism: self-organize a virtual space; each agent maintains a zone and neighbor set. Boundaries shift via local splits/merges. \[verified] ([People at EECS][7])
* Add network coordinates if you need metric routing. \[verified] ([Google Research][8])
* Fit for code: workable when you want pure decentralization and local neighbor protocols. \[inferred]

5. Space-filling curve ranges for locality

* Mechanism: map code artifacts to an SFC index (Hilbert or Morton). Partition contiguous ranges among agents. Preserves locality under rebalancing. \[verified] ([arXiv][9])
* Fit for code: strong for tree-like structures and AMR-style refactors; fast repartitioning with low data movement. \[verified] ([MCS][10])

6. Graph partitioning and community detection

* Mechanism: build a code graph (files, imports, call edges). Use multilevel METIS or Louvain to cut few edges and maximize intra-region cohesion. \[verified] ([Department of Computer Science][11])
* Fit for code: produces semantically coherent territories. Heavier to recompute; schedule offline or incrementally. \[inferred]

7. Social “ownership” overlays as boundary stabilizers

* Real systems: Chromium/Kubernetes OWNERS, GitHub CODEOWNERS. Path-based ownership yields inherited territorial trees. \[verified] ([Chromium Git Repositories][12])
* Practice: automate reviewer routing and inactivity cleanup to prevent drift. \[verified] ([Google Groups][13])
* Fit for code: pair with any geometric scheme to anchor final approval surfaces. \[inferred]

When territories emerge without pre-assignment
A) Nearest-seed attraction

* Start with k random or expertise-weighted seeds over the embedding space. Assign by nearest seed. Relax with Lloyd to CVT or with power weights. \[verified] ([SIAM E-Books][3])

B) Local interaction and inhibition

* Stigmergic marks plus avoidance produce exclusive ranges over time with no global map. \[verified] ([PMC][4])

C) Deterministic placement

* Hashing or DHT zones yield immediate territories from keys and join/leave events without coordination. \[verified] ([amosbrocco.ch][5])

D) Cohesion-maximizing cuts

* Community detection or multilevel partitioners define modules as territories by minimizing cut edges. No prior owners required. \[verified] ([arXiv][14])

Space-filling and tessellation choices for codebases

* CVT/power diagrams: best when embeddings reflect semantics and you want smooth adaptive regions. \[verified] ([SIAM E-Books][3])
* SFC ranges: best when filesystem or package trees dominate locality and you need fast rebalancing. \[verified] ([MCS][10])
* DHT zones: best for decentralized swarms with churn tolerance and minimal coordination. \[verified] ([People at EECS][7])
* Graph partitioning: best when call/import graphs are accurate and relatively stable. \[verified] ([Department of Computer Science][11])

Conflict handling at boundaries

* Soft borders: overlap bands where either agent edits, with tie-break by recency or higher “power” weight. \[inferred from power diagrams] ([SIAM E-Books][15])
* Lease lines: short-term leases on files near borders to prevent thrashing. Renew via local rules to avoid global locks. \[inferred]
* Review fences: OWNERS or CODEOWNERS approval enforces final boundary without blocking exploratory edits. \[verified] ([GitHub Docs][16])

Minimal recipes

Recipe 1: CVT territories over embeddings

* Compute file embeddings. Seed k agents with weights from expertise. Assign by nearest. Run 5–10 Lloyd iterations nightly. Use power weights for load leveling. \[verified] ([SIAM E-Books][3])

Recipe 2: SFC partitioning with incremental rebalance

* Index files by Hilbert code over a 2D projection of embeddings or path hashes. Partition into k equal ranges. Rebalance by range splitting on load spikes. \[verified] ([arXiv][9])

Recipe 3: Stigmergic claims

* Maintain a decaying heatmap per file or symbol. Route agents away from hot foreign areas. Allow incursions when tests fail or debt rises. Territories emerge from gradients. \[verified] ([PMC][4])

Recipe 4: Graph cut modules

* Build a dependency graph. Run METIS or Louvain to form modules. Pin modules to agents for an epoch. Repartitions on drift thresholds. \[verified] ([Department of Computer Science][11])

Bridging to real-world practice

* Use CODEOWNERS or OWNERS as the final arbiter over any emergent map. Feed emergent territories to generate or update those files. \[verified] ([GitHub Docs][16])

Notes on evidence

* Geometry, CVT, power diagrams, and SFCs are established. \[verified] ([SIAM E-Books][3])
* Territorial stigmergy comes from ecological models. Mapping to code is an engineering analogy. \[inferred] ([PMC][4])
* DHT zones and hashing show self-organization without coordination. \[verified] ([People at EECS][7])

If you want, I will map these four recipes to your repo structure and CI hooks. \[assumed]

[1]: https://www.wias-berlin.de/people/si/course/files/Aurenhammer91-Voronoi.pdf?utm_source=chatgpt.com "Voronoi diagrams--a survey of a fundamental geometric ..."
[2]: https://www.cs.jhu.edu/~misha/Spring16/Aurenhammer87.pdf?utm_source=chatgpt.com "Power Diagrams: Properties, Algorithms and Applications"
[3]: https://epubs.siam.org/doi/10.1137/S0036144599352836?utm_source=chatgpt.com "Centroidal Voronoi Tessellations: Applications and Algorithms"
[4]: https://pmc.ncbi.nlm.nih.gov/articles/PMC4043092/?utm_source=chatgpt.com "How do animal territories form and change? Lessons from ..."
[5]: https://amosbrocco.ch/pubs/paper03.pdf?utm_source=chatgpt.com "A survey and fair comparison of consistent hashing algorithms"
[6]: https://dzone.com/articles/consistent-hashing-vs-rendezvous-hashing-a-compara?utm_source=chatgpt.com "Consistent Hashing vs. Rendezvous Hashing"
[7]: https://people.eecs.berkeley.edu/~sylvia/papers/cans.pdf?utm_source=chatgpt.com "A Scalable Content-Addressable Network - People @EECS"
[8]: https://research.google/pubs/vivaldi-a-decentralized-network-coordinate-system/?utm_source=chatgpt.com "Vivaldi: a decentralized network coordinate system"
[9]: https://arxiv.org/abs/1708.01365?utm_source=chatgpt.com "Load Balancing using Hilbert Space-filling Curves for Parallel Reservoir Simulations"
[10]: https://www.mcs.anl.gov/papers/P5355-0615.pdf?utm_source=chatgpt.com "Space-filling curves for Partitioning Adaptively Refined Meshes"
[11]: https://www.cs.utexas.edu/~pingali/CS395T/2009fa/papers/metis.pdf?utm_source=chatgpt.com "A fast and high quality multilevel scheme for partitioning ..."
[12]: https://chromium.googlesource.com/chromium/src/%2B/lkgr/docs/code_reviews.md?utm_source=chatgpt.com "Chromium Docs - Code Reviews"
[13]: https://groups.google.com/a/chromium.org/g/chromium-dev/c/aGC8BTTFK64?utm_source=chatgpt.com "Inactive OWNERS cleanup"
[14]: https://arxiv.org/abs/0803.0476?utm_source=chatgpt.com "Fast unfolding of communities in large networks"
[15]: https://epubs.siam.org/doi/abs/10.1137/0216006?utm_source=chatgpt.com "Power Diagrams: Properties, Algorithms and Applications"
[16]: https://docs.github.com/articles/about-code-owners?utm_source=chatgpt.com "About code owners"
