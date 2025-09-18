# Implementation-Focused Research Report: Cognitive Architectures and Consciousness

**Author:** Manus AI

**Date:** September 2, 2025

## Introduction

This report provides a comprehensive overview of the current landscape of cognitive architectures and consciousness research, with a specific focus on implementation details, emerging projects, and practical approaches. The research covers a wide range of topics, including GitHub repositories, memory and forgetting mechanisms, parallel processing, emotion and motivation systems, small team and independent projects, benchmarks and evaluation metrics, and resource-efficient implementations. The findings are based on a thorough review of recent academic papers, technical articles, and industry reports, providing a detailed snapshot of the state-of-the-art in this rapidly evolving field.






## GitHub Repositories and Code Implementations

### ACT-R Implementations

- **PyACTR**: A Python package for creating and running ACT-R cognitive models, supporting both symbolic and subsymbolic processes. It is available on GitHub at [jakdot/pyactr](https://github.com/jakdot/pyactr).
- **Python ACT-R**: An alternative Python implementation from the Carleton Cognitive Modeling Lab, available at [CarletonCognitiveModelingLab/python_actr](https://github.com/CarletonCognitiveModelingLab/python_actr). Note that this version does not support Python 3.12 or higher.
- **GACTAR**: A tool for creating and running basic ACT-R models on multiple implementations using a single declarative file format, available at [asmaloney/gactar](https://github.com/asmaloney/gactar).
- **jACT-R**: A Java-based implementation with tutorials available at [amharrison/jactr-tutorials](https://github.com/amharrison/jactr-tutorials).

### LIDA Cognitive Architecture Implementations

- **Private Machine**: An AI companion system with emotion, needs, and goals simulation based on the LIDA cognitive architecture. More information can be found on GitHub under the [lida-cognitive-architecture](https://github.com/topics/lida-cognitive-architecture) topic.
- **CST (Cognitive Systems Toolkit)**: A Java-based toolkit for constructing Cognitive Architectures, developed at the University of Campinas. The repository is available at [CST-Group/cst](https://github.com/CST-Group/cst).
- **LIDA Framework**: The official framework from the Cognitive Computing Research Group (CCRG) at the University of Memphis, providing a generic and configurable version of the LIDA modules and processes. More information is available at the [CCRG website](https://ccrg.cs.memphis.edu/framework.html).

### Active Inference Implementations

- **PyMDP**: The primary Python package for simulating Active Inference agents in Markov Decision Process environments. It is available on GitHub at [infer-actively/pymdp](https://github.com/infer-actively/pymdp).
- **cpp-AIF**: A multi-core C++ implementation of Active Inference for Partially Observable Markov Decision Processes, available as a header-only library. More information can be found in the Neurocomputing journal (February 1, 2024).

### Predictive Coding Implementations

- **Predify**: An open-source PyTorch package for adding predictive coding dynamics to existing deep neural networks. The repository is available at [miladmozafari/predify](https://github.com/miladmozafari/predify).
- **PC-DARTS**: A memory-efficient differentiable architecture method based on DARTS, with the original implementation available at [yuhuixu1993/PC-DARTS](https://github.com/yuhuixu1993/PC-DARTS).

### Global Workspace Theory Implementations

- **GW-MoE**: An official implementation of "GW-MoE: Resolving Uncertainty in MoE Router with Global Workspace Theory," applying GWT to Mixture of Experts architectures. The repository is available at [WaitHZ/GW-MoE](https://github.com/WaitHZ/GW-MoE).
- **Legion AGI**: A multi-agent, reasoning-based artificial intelligence system with GWT support for integrating multiple cognitive processes. The repository is available at [dotdigitize/legion_agi](https://github.com/dotdigitize/legion_agi).

### Conscious Turing Machine Implementations

- **CTM Implementation**: A toy implementation of Lenore and Manuel Blum's model, available at [cvaisnor/conscious_turing_machine](https://github.com/cvaisnor/conscious_turing_machine).
- **Wolfram Implementation**: An implementation of the dynamics of Conscious Turing Machines, available on the Wolfram Cloud.






## Memory and Forgetting Mechanisms

### Complementary Learning Systems (CLS)

- **Theoretical Foundation**: The CLS theory proposes that the brain uses two interacting systems for learning: the hippocampus for rapid learning of new information and the neocortex for gradual integration of knowledge. This division of labor allows the brain to learn efficiently without disrupting existing knowledge structures.
- **Key Publications**:
  - **McClelland, McNaughton, O'Reilly (1995)**: Foundational paper on CLS theory.
  - **Nature Neuroscience (2023)**: Recent advances in CLS, focusing on memory organization for generalization.
  - **PMC (2024)**: A hippocampus-inspired approach to the stability-plasticity dilemma.

### Sleep-Wake Consolidation Algorithms

- **Wake-Sleep Consolidated Learning (WSCL)**: A learning strategy inspired by the human wake-sleep cycle, leveraging CLS theory for artificial neural networks.
- **Sleep-Like Unsupervised Replay**: A method that uses spontaneous replay during simulated sleep phases to reduce catastrophic forgetting in neural networks.

### Catastrophic Forgetting Solutions

- **Elastic Weight Consolidation (EWC)**: A regularization technique that protects knowledge of previous tasks during new learning.
- **PackNet**: An iterative pruning approach that adds multiple tasks to a single network by freeing up redundant parameters.
- **Progressive Neural Networks**: A method that adds new neural network columns for each new task, preventing catastrophic forgetting but at the cost of linear network growth.

### Experience Replay Mechanisms

- **Prioritized Experience Replay (PER)**: A framework that prioritizes important transitions for replay, improving the performance of Deep Q-Networks (DQN).
- **Advanced Techniques**: Recent research has explored prioritized generative replay and adaptive experience replay to further enhance learning efficiency.

### Hebbian Learning Implementations

- **Theoretical Foundation**: Based on the principle that "cells that fire together, wire together," Hebbian learning provides a biologically plausible alternative to backpropagation.
- **Implementations**: Various implementations exist, including unsupervised Hebbian learning, deep neural networks with Hebbian learning, and specialized versions like Spike-Timing-Dependent Plasticity (STDP) and Oja's Rule.






## Parallel Processing and Multi-Stream Architectures

### Multi-Agent Cognitive Architectures

- **Concurrent Processing**: Modern cognitive architectures are increasingly designed with agentic workflow patterns that allow specialized sub-agents to execute tasks independently and in parallel. This approach is exemplified in Anthropic's multi-agent research system and Google Cloud's "Designing Cognitive Architectures" article.
- **Historical Context**: Early research, such as the DUAL and RCS cognitive architectures, explored hybrid multi-agent approaches to cognitive modeling, laying the groundwork for today's more advanced systems.

### Mixture of Experts (MoE) for Cognitive Modeling

- **Brain-Inspired Models**: MoE architectures are inspired by the brain's ability to weight contributions from different expert systems. This approach is being explored in AI to enhance the cognitive depth of language models.
- **Implementations**: Resources like the "awesome-mixture-of-experts" GitHub repository and NVIDIA's guide on applying MoE in LLM architectures provide valuable insights into implementing these models.

### Asynchronous Neural Network Training

- **Asynchronous Methods**: Asynchronous training methods, such as asynchronous deep reinforcement learning and GPU asynchronous SGD, are used to accelerate neural network training by leveraging parallel processing.
- **Advanced Systems**: Recent research has focused on developing distributed asynchronous optimization (DASO) and other advanced methods to further improve training efficiency.

### Stream Processing Architectures

- **Cognitive Stream Processing**: Stream production systems, as described in a 2024 ScienceDirect paper, enable rapid processing of all incoming information, a key feature for real-time cognitive systems.
- **Infrastructure**: Modern data streaming architectures, as detailed by Estuary and Tinybird, provide the necessary infrastructure for handling high-volume data streams in cognitive applications.

### System 1 / System 2 Dual Processing

- **Theoretical Foundation**: Based on Daniel Kahneman's dual-process theory, this approach models two types of thinking: System 1 (fast, intuitive) and System 2 (slow, deliberate).
- **AI Implementations**: Projects like SOFAI (Slow and Fast AI) and research on integrating System 1 and System 2 in LLMs are exploring how to implement this dual-processing model in AI systems.






## Emotion and Motivation Systems

### Computational Models of Emotion

- **OCC Model (Ortony, Clore, and Collins)**: A widely used model in AI for classifying emotions into 22 types based on the consequences of events, actions of agents, and aspects of objects. Implementations include visual character emotion emulators and logic-based emotion modeling.
- **WASABI (WASABI Affect Simulation for Agents with Believable Interactivity)**: A computational architecture for creating emotionally believable agents, with a focus on dynamic emotional responses.
- **EMA (Emotion and Adaptation)**: A process model of appraisal dynamics that explains both rapid and slow emotional reactions.

### Intrinsic Motivation in AI

- **Curiosity-Driven Learning**: A key area of research in AI, where artificial curiosity provides a natural intrinsic motivation for efficient learning. OpenAI has conducted large-scale studies on this topic, and researchers at MIT and the University of Chicago are developing algorithms to control and leverage curiosity in AI systems.
- **Empowerment**: An alternative approach to intrinsic motivation where agents are driven to maximize their influence on the environment.

### Drive and Homeostasis Implementations

- **AI-Driven Homeostasis**: The concept of implementing homeostatic mechanisms in AI to create more stable, efficient, and sustainable systems. This approach is being explored for applications in AI alignment, emotional AI, and sustainability.
- **Biological Inspiration**: Research is drawing inspiration from biological systems, such as self-concern and circadian rhythms, to create more human-like AI.

### Somatic Marker Hypothesis Implementations

- **Theoretical Foundation**: This hypothesis proposes that bodily emotional signals (somatic markers) play a crucial role in decision-making. It is being explored as a mechanism for creating more intuitive and human-like AI.
- **Computational Models**: Researchers are developing computational models of the somatic marker hypothesis to integrate emotional signals into AI decision-making processes.






## Small Team and Independent Projects (2024-2025)

### Recent Cognitive Architecture Projects

- **The Cognitive Core**: An integrated cognitive architecture that combines high-level reasoning and associative memory, with a recent publication in June 2025.
- **Adaptive Cognitive Architecture for ML Monitoring**: A project from June 2025 that focuses on creating an adaptive cognitive architecture for interpretable machine learning monitoring.
- **Hybrid Tool for Human-Centered Architectural Design**: A project from June 2025 that combines Christopher Alexander's Pattern Language with generative AI for architectural design.

### University Research Groups and Academic Projects

- **University of Manchester**: Developing a cognitive architecture for robotic agents based on the Global Workspace Theory.
- **University of Cambridge**: Conducting comprehensive research across multiple domains of cognitive architecture.
- **Princeton Laboratory for Artificial Intelligence**: A new lab launched in Fall 2024 to support interdisciplinary AI research.
- **University of Bern**: Developing a new model for the emergence of consciousness, with a focus on pain-free AI.

### Open-Source Initiatives and Individual Researchers

- **Soar Cognitive Architecture**: A general cognitive architecture that is open source and freely available.
- **System 2 Reasoning Research Collection**: A GitHub repository that collects materials related to reasoning and cognition in AI systems.
- **The Brain System**: A self-evolving cognitive architecture for human-AI collaboration, developed by an individual researcher.

### Startup Companies and Commercial Initiatives

- **Conscium**: A company pioneering safe and efficient AI, with a focus on machine consciousness research.
- **Cognition AI**: The creators of Devin, an AI agent capable of performing complex tasks autonomously.
- **Eleos AI Research**: An organization investigating potential AI consciousness and welfare.






## Benchmarks, Evaluation Metrics, and Practical Guides

### Consciousness Assessment Tools and Metrics

- **AI for Clinical Consciousness Detection**: Recent studies have shown that AI can detect signs of covert consciousness in comatose patients, sometimes even before doctors. This has significant implications for clinical assessment and prognosis.
- **Consciousness Measurement Frameworks**: Researchers are developing frameworks for the scientific study of consciousness, including behavioral and neurophysiological measures, to provide a more structured approach to consciousness assessment.
- **EEG-Based Measures**: A systematic review of 255 EEG-based measures of consciousness has been conducted to identify the most effective methods for distinguishing between conscious and unconscious states.

### AI Consciousness Assessment Methodologies

- **Rigorous Empirical Approach**: A 2023 report from a team of 19 neuroscientists, philosophers, and computer scientists provides a rigorous and empirically grounded approach to AI consciousness assessment. The report concludes that no current AI systems are conscious but provides tools for future assessment.
- **Testing Protocols**: Researchers are developing AI consciousness tests (ACTs) to assess whether AI systems can spontaneously understand and use concepts about their internal experiences.

### Cognitive Architecture Evaluation Frameworks

- **Design and Evaluation Criteria**: Researchers are developing criteria for the design and evaluation of cognitive architectures, with a focus on creating frameworks that facilitate the evaluation of their effectiveness.
- **Neurosymbolic Frameworks**: A novel cognitive architecture framework that combines symbolic planning and counterfactual reasoning is being developed to handle open-world novelties.

### Performance Metrics for Cognitive Systems

- **Cognitive Performance Monitoring**: Researchers are developing strategies for monitoring cognitive performance using a combination of EEG measures, performance metrics, and other physiological indicators.
- **Cognitive Load Metrics**: A survey of cognitive load metrics has been conducted to identify the most effective methods for assessing cognitive load in various settings.

### Implementation Guides and Tutorials

- **ICARUS Cognitive Architecture Tutorial**: A tutorial on the ICARUS cognitive architecture is available, providing practical guidance on its implementation.
- **ACT-R Step-by-Step Tutorial**: A step-by-step tutorial on using the ACT-R cognitive architecture is available, with data and computer code for replication.
- **Soar Cognitive Architecture Introduction**: A comprehensive tutorial on the Soar cognitive architecture is available from Penn State University.






## Resource-Efficient Implementations and Commercial Activity

### Lightweight Cognitive Architectures

- **Biomimetic Models**: The extended ramp model provides a biomimetic approach to behavior arbitration for lightweight cognitive architectures, suitable for individual agents or swarms.
- **Game Development**: Lightweight cognitive architectures are being used in game development to create non-player characters (NPCs) with emotions and believable behavior.
- **Symbolic Deep Networks**: Researchers are developing psychologically inspired lightweight cognitive architectures that integrate state recognition, action-selection, and hierarchical memory systems.

### Edge Computing and Cognitive AI

- **Cognitive Edge Computing**: There is a growing trend of moving intelligence to the edge of the network, enabling real-time data processing and analysis without reliance on the cloud.
- **Applications**: Edge AI is being used in a wide range of applications, including IoT devices, autonomous vehicles, and smart manufacturing.

### Resource-Conscious Computing

- **Efficient AI**: Researchers are exploring ways to leverage conscious and nonconscious learning to create more efficient AI systems.
- **Energy Efficiency**: Resource-conscious scheduling and power-conscious systems are being developed to improve the energy efficiency of AI.

### Commercial Cognitive AI Products

- **Top AI Products**: The market for AI products is growing rapidly, with a wide range of products available for various applications.
- **AI-Driven Cognitive Applications**: The market for AI-driven cognitive applications is expected to reach $10.35 billion by 2030.

### Startup Funding and Investment Activity

- **Major Funding Rounds**: Several startups in the cognitive AI space have recently raised significant funding, including Sanctuary Cognitive Systems and Cognition AI.
- **Investment Trends**: There is a high adoption rate of AI in the investment management sector, with 91% of managers currently using or planning to use AI.






## Conclusion

The research conducted in this report highlights the significant progress being made in the field of cognitive architectures and consciousness. From the development of more sophisticated and resource-efficient implementations to the growing commercial interest and investment in this area, it is clear that cognitive AI is poised for rapid advancement. The increasing availability of open-source tools, practical guides, and evaluation frameworks is also helping to accelerate research and development in this field.

Looking ahead, we can expect to see continued growth in the development of multi-agent systems, brain-inspired architectures, and AI with more human-like emotional and motivational capabilities. As the field matures, we can also expect to see a greater focus on the ethical implications of AI consciousness and the development of more robust safety and alignment protocols.

## References

- [jakdot/pyactr](https://github.com/jakdot/pyactr)
- [CarletonCognitiveModelingLab/python_actr](https://github.com/CarletonCognitiveModelingLab/python_actr)
- [asmaloney/gactar](https://github.com/asmaloney/gactar)
- [amharrison/jactr-tutorials](https://github.com/amharrison/jactr-tutorials)
- [lida-cognitive-architecture](https://github.com/topics/lida-cognitive-architecture)
- [CST-Group/cst](https://github.com/CST-Group/cst)
- [CCRG website](https://ccrg.cs.memphis.edu/framework.html)
- [infer-actively/pymdp](https://github.com/infer-actively/pymdp)
- [miladmozafari/predify](https://github.com/miladmozafari/predify)
- [yuhuixu1993/PC-DARTS](https://github.com/yuhuixu1993/PC-DARTS)
- [WaitHZ/GW-MoE](https://github.com/WaitHZ/GW-MoE)
- [dotdigitize/legion_agi](https://github.com/dotdigitize/legion_agi)
- [cvaisnor/conscious_turing_machine](https://github.com/cvaisnor/conscious_turing_machine)
- [McClelland, McNaughton, O'Reilly (1995)](https://stanford.edu/~jlmcc/papers/McCMcNaughtonOReilly95.pdf)
- [Nature Neuroscience (2023)](https://www.nature.com/articles/s41593-023-01382-9)
- [PMC (2024)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11591613/)
- [SmythOS](https://smythos.com/developers/agent-development/cognitive-agent-architectures/)
- [Medium/Google Cloud (October 24, 2024)](https://medium.com/google-cloud/designing-cognitive-architectures-agentic-workflow-patterns-from-scratch-63baa74c54bc)
- [Anthropic (June 13, 2025)](https://www.anthropic.com/engineering/built-multi-agent-research-system)
- [The Decision Lab](https://thedecisionlab.com/reference-guide/philosophy/system-1-and-system-2-thinking)
- [LessWrong (March 31, 2023)](https://www.lesswrong.com/w/dual-process-theory-system-1-and-system-2)
- [Google Sites](https://sites.google.com/view/sofai/home)
- [The Cognitive Core: An Integrated Cognitive Architecture](https://www.researchgate.net/publication/392774960_The_Cognitive_Core_An_Integrated_Cognitive_Architecture)
- [An Adaptive Cognitive Architecture for Interpretable ML Monitoring](https://arxiv.org/html/2506.09742v1)
- [A Hybrid Tool for Human-Centered Architectural Design](https://www.preprints.org/manuscript/202505.2258/v2/download)
- [Cognitive Architecture for Social Perception and Engagement in Human-Robot Interaction](https://link.springer.com/article/10.1007/s12369-024-01116-2)
- [Socially adaptive cognitive architecture for human-robot collaboration in industrial settings](https://www.frontiersin.org/journals/robotics-and-ai/articles/10.3389/frobt.2024.1248646/full)
- [Synthetic consciousness architecture](https://www.frontiersin.org/journals/robotics-and-ai/articles/10.3389/frobt.2024.1437496/full)
- [University of Manchester](https://research.manchester.ac.uk/en/studentTheses/incremental-development-of-a-cognitive-architecture-based-on-the-)
- [University of Cambridge](https://www.cca.arct.cam.ac.uk/)
- [Princeton University](https://dof.princeton.edu/news/2024/princeton-laboratory-artificial-intelligence-stretch-horizons-ai-research-faculty-and-researchers)
- [University of Bern](https://mediarelations.unibe.ch/media_releases/2024/media_releases_2024/ai_with_consciousness___but_pain_free/index_eng.html)
- [CRAM 2.0 Robot Cognitive Architecture](http://vernon.eu/publications/2025_Beetz_et_al.pdf)
- [Soar Cognitive Architecture](https://soar.eecs.umich.edu/)
- [open-thought/system-2-research](https://github.com/open-thought/system-2-research)
- [The Brain System: A Self-Evolving Cognitive Architecture for Human-AI Collaboration](https://medium.com/@mbonsign/the-brain-system-a-self-evolving-cognitive-architecture-for-human-ai-collaboration-849f4511df18)
- [Kenny Bastani's 2024 Work](https://www.kennybastani.com/2024/?m=0)
- [Applying Cognitive Design Patterns to General LLM Agents](https://arxiv.org/html/2505.07087v2)
- [Conscium](https://conscium.com/)
- [Venture Atlanta Top Startup Industries 2025](https://www.ventureatlanta.org/top-startup-industries-2025/)
- [Eleos AI Research](https://eleosai.org/research/)
- [CogArch 2024](https://research.ibm.com/publications/cogarch-2024-8th-workshop-on-cognitive-architectures)
- [Models of Consciousness 2024](https://amcs-community.org/events/moc5-2024/)
- [Consciousness in Cognitive Systems](https://www.youtube.com/watch?v=4hnnV5E2ghg)
- [Beyond the Benchmarks: Deconstructing the Cognitive Architecture of LLMs to Forge a New Path Toward AGI](https://medium.com/@adnanmasood/beyond-the-benchmarks-deconstructing-the-cognitive-architecture-of-llms-to-forge-a-new-path-toward-ec22c21684e5)
- [Temporal heterogeneity in cognitive architectures](https://www.sciencedirect.com/science/article/pii/S1389041724000597)
- [Cognitive architecture for cognitive cyber-physical systems](https://www.sciencedirect.com/science/article/abs/pii/S2405896324015027)
- [The people who think AI might become conscious](https://www.bbc.com/news/articles/c0k3700zljjo)
- [Signs of consciousness in AI: Can GPT-3 tell how smart it really is?](https://www.nature.com/articles/s41599-024-04154-3)
- [How to Detect Consciousness in People, Animals and Maybe Even AI](https://www.scientificamerican.com/article/how-to-detect-consciousness-in-people-animals-and-maybe-even-ai/)
- [Principles for responsible AI consciousness research](https://www.jair.org/index.php/jair/article/view/17310)
- [AI consciousness and public perceptions: Four futures](https://arxiv.org/abs/2408.04771)
- [The extended ramp model: A biomimetic model of behaviour arbitration for lightweight cognitive architectures](https://www.sciencedirect.com/science/article/pii/S1389041717300979)
- [How Behaviour Trees and a Lightweight Cognitive Architecture Enable the Development of Non-Player Characters with Emotions](https://equis.cs.queensu.ca/~equis/pubs/2019/belle-gem-19.pdf)
- [Symbolic Deep Networks: A Psychologically Inspired Lightweight Cognitive Architecture](https://www.jstor.org/stable/community.34434811)
- [Cognitive Architectures are likely to be the next big step forward](https://www.reddit.com/r/singularity/comments/1984wb8/cognitive_architectures_are_likely_to_be_the_next/)
- [A lightweight framework for perception analysis based on multimodal cognition-aware computing](https://www.frontiersin.org/journals/neuroscience/articles/10.3389/fnins.2022.879348/full)
- [Cognitive edge computing through artificial intelligence](https://ieeexplore.ieee.org/document/9142010/)
- [What Is Edge AI?](https://www.ibm.com/think/topics/edge-ai)
- [Introducing the Cognitive Computing Continuum Cluster Projects](https://eucloudedgeiot.eu/introducing-the-cognitive-computing-continuum-cluster-projects-pioneering-the-future-of-ai-and-edge-computing/)
- [Edge Computing and AI: The Future of Real-Time Data Processing](https://www.sapien.io/blog/edge-computing-and-ai)
- [AI and the Edge: Bringing Intelligence Closer to Data](https://resolvetech.com/ai-and-the-edge-bringing-intelligence-closer-to-data/)
- [At the confluence of artificial intelligence and edge computing in IoT-based applications: A review and new perspectives](https://www.mdpi.com/1424-8220/23/3/1639)
- [iRobot-Factory: An intelligent robot factory based on cognitive manufacturing and edge computing](https://www.sciencedirect.com/science/article/pii/S0167739X1831183X)
- [Leveraging conscious and nonconscious learning for efficient AI](https://pmc.ncbi.nlm.nih.gov/articles/PMC10076654/)
- [Implications of resource limitations for a conscious machine](https://www.sciencedirect.com/science/article/abs/pii/S0925231208004669)
- [Resource-conscious scheduling for energy efficiency on multicore processors](https://dl.acm.org/doi/abs/10.1145/1755913.1755930)
- [Software implementation strategies for power-conscious systems](https://link.springer.com/article/10.1023/A:1011487018981)
- [Consciousness and Energy Processing in Neural Systems](https://pmc.ncbi.nlm.nih.gov/articles/PMC11591782/)
- [Top 10 AI Products to use in 2024](https://www.analyticsvidhya.com/blog/2024/03/top-ai-products-to-use/)
- [The 10 Coolest GenAI Products And AI Tools Of 2024](https://www.crn.com/news/ai/2024/the-10-coolest-genai-products-and-ai-tools-of-2024)
- [15 Most Important AI Products of 2024](https://www.youtube.com/watch?v=h6ISfS5rupw)
- [Best 10 Artificial Intelligence Platforms for Business of 2024](https://medium.com/brilworks-engineering/best-10-artificial-intelligence-platforms-for-business-of-2024-e5c515c53330)
- [AI-Driven Cognitive Applications Q4 2024](https://www.auditoria.ai/report-ai-driven-cognitive-applications-q4-2024/)
- [Agentic AI will revolutionize business in the cognitive era](https://www.weforum.org/stories/2025/06/cognitive-enterprise-agentic-business-revolution/)
- [Unlocking AI Consciousness to Transform Ecommerce Marketing](https://bermont.digital/blog/unlocking-ai-consciousness-to-transform-ecommerce-marketing)
- [Sanctuary Cognitive Systems Closes C$75.5 Million (US$58.5 Million) Series A Funding](https://www.sanctuary.ai/blog/sanctuary-ai-closes-75-million-series-a-funding)
- [AI coding assistant startup Cognition reportedly raises nearly $500M at $9.8B valuation](https://siliconangle.com/2025/08/14/ai-coding-assistant-startup-cognition-reportedly-raises-nearly-500m-9-8b-valuation/)
- [Cognition Cinches About $500 Million to Advance AI Code Generation Business](https://www.wsj.com/articles/cognition-cinches-about-500-million-to-advance-ai-code-generation-business-f65f71a9)
- [QbiqAI raises $16M to automate architectural design and visualization with AI](https://siliconangle.com/2025/01/15/qbiq-raises-16m-automate-architectural-design-visualization-ai/)
- [AI in investment management survey 2024](https://www.mercer.com/insights/investments/portfolio-strategies/ai-in-investment-management-survey/)
- [ARTIFICIAL INTELLIGENCE: Investment Opportunities and Challenges](https://www.theia.org/sites/default/files/2024-10/Technology%20Working%20Group%20AI%20Report%20Oct%202024.pdf)
- [The 2025 AI Index Report](https://hai.stanford.edu/ai-index/2025-ai-index-report)
- [IDC's 2024 AI opportunity study: Top five AI trends to watch](https://blogs.microsoft.com/blog/2024/11/12/idcs-2024-ai-opportunity-study-top-five-ai-trends-to-watch/)
- [The State of AI: Global survey](https://www.mckinsey.com/capabilities/quantumblack/our-insights/the-state-of-ai)
- [2024 Global Trends in AI](https://www.weka.io/resources/analyst-report/2024-global-trends-in-ai/)
- [Enterprise AI is at a tipping Point, here's what comes next](https://www.weforum.org/stories/2025/07/enterprise-ai-tipping-point-what-comes-next/)
- [AI in the Enterprise](https://cdn.openai.com/business-guides-and-resources/ai-in-the-enterprise.pdf)
- [AI in the workplace: A report for 2025](https://www.mckinsey.com/capabilities/mckinsey-digital/our-insights/superagency-in-the-workplace-empowering-people-to-unlock-ais-full-potential-at-work)
- [Usage of Cognitive Architectures in the Development of Industrial Applications](https://www.scitepress.org/papers/2018/66798/66798.pdf)
- [Deep learning based robot cognitive architecture for collaborative manufacturing](https://www.sciencedirect.com/science/article/pii/S0736584523000480)
- [A cognitive model for technology adoption](https://www.mdpi.com/1999-4893/16/3/155)
- [A review of 40 years of cognitive architecture research: Focus on perception, attention, learning and applications](https://www.researchgate.net/profile/Oscar-Avella-2/publication/309483878_A_Review_of_40_Years_of_Cognitive_Architecture_Research_Focus_on_Perception_Attention_Learning_and_Applications/links/67b2782c207c0c20fa8bbc29/A-Review-of-40-Years-of-Cognitive-Architecture-Research-Focus-on-Perception-Attention-Learning-and-Applications.pdf)
- [An overview on the evolution and adoption of deep learning applications used in the industry](https://wires.onlinelibrary.wiley.com/doi/abs/10.1002/widm.1257)
- [A cognition-driven framework for the evaluation of startups in the digital economy](https://www.emerald.com/insight/content/doi/10.1108/md-09-2019-1253/full/html)
- [Cognitive modeling of the startup life cycle](https://cyberleninka.ru/article/n/cognitive-modeling-of-the-startup-life-cycle)
- [How Entrepreneurs make sense of Lean Startup Approaches: Business Models as cognitive lenses](https://www.sciencedirect.com/science/article/pii/S0040162520311501)
- [Strong artificial intelligence and consciousness](https://www.worldscientific.com/doi/abs/10.1142/S2705078520300042)
- [Artificial intelligence: consciousness and conscience](https://link.springer.com/article/10.1007/s00146-019-00880-4)
- [Future Human: Consciousness, Cognition and the Role of Human Insight in an AI Future](https://books.google.com/books?hl=en&lr=&id=24xBEQAAQBAJ&oi=fnd&pg=PR7&dq=AI+consciousness+investment+trends+2024&ots=K4wPMei1lc&sig=T3J7E9tignIqB3WB-COluwcGRFg)
- [Artificial Intelligence (AI) in modern financial practices and education](https://www.ceeol.com/search/article-detail?id=1303871)



