# Uncan V8 Physics Engine: Research Report

## Introduction

This report summarizes the findings of a comprehensive research investigation into the technical feasibility and implementation details of the Uncan V8 physics engine. The research covers a wide range of topics, including WebGPU integration, performance optimization, visualization techniques, and specific implementation questions. The goal of this research is to provide a solid foundation for the development of the Uncan V8 physics engine, a next-generation physics engine designed for the web.





## 1. PixiJS 8.5+ and WebGPU Buffer Sharing

### 1.1. Key Findings

The integration of custom WebGPU compute pipelines with a rendering library like PixiJS is a cornerstone of the Uncan V8 engine. Our research indicates that PixiJS version 8.12.0 and later provides the necessary capabilities to achieve this integration seamlessly. The key is the ability to initialize a PixiJS application with a pre-existing `GPUAdapter` and `GPUDevice`. This allows for the sharing of the WebGPU context between the custom compute shaders and the PixiJS rendering engine, which is essential for efficient data transfer and a high-performance architecture.

The `BufferSource` class in PixiJS is the primary mechanism for sharing data from a WebGPU compute pipeline to the rendering engine. This class allows the creation of a PixiJS `Texture` directly from a `GPUBuffer`. This is the ideal method for visualizing the output of a compute shader, as it avoids the costly process of reading data back from the GPU to the CPU. The workflow is straightforward: a compute shader writes its output to a `GPUBuffer`, a `BufferSource` is created to wrap this buffer, and then a `Texture` is generated from the `BufferSource`. This texture can then be used in any PixiJS display object, such as a `Sprite`, for rendering.

Synchronization between the compute and rendering passes is another critical aspect. While PixiJS does not provide its own high-level synchronization primitives, standard WebGPU synchronization techniques are fully applicable. The `GPUQueue.onSubmittedWorkDone()` method, which returns a promise that resolves when the submitted work is complete, is a suitable mechanism for ensuring that the compute pass has finished before the rendering pass attempts to use the output buffer. This prevents race conditions and ensures that the rendered data is always up-to-date.

### 1.2. Further Research Needed

While the documentation provides the theoretical foundation for this integration, practical, real-world examples are needed to solidify our understanding and provide a clear implementation path. We will focus on finding or creating a complete example of sharing a buffer between a custom WebGPU compute pipeline and the PixiJS rendering engine. This will involve a deep dive into community forums, open-source projects, and tutorials to find a suitable example.

Furthermore, we need to investigate the specific implementation details of visualizing heat textures and rendering field gradients. These are direct applications of the buffer sharing technique, and a deeper understanding of the data formats and shader techniques required will be necessary. This will involve experimenting with different data layouts and shader effects to achieve the desired visual results.





## 2. Rapier2D Force Application from GPU Fields

### 2.1. Key Findings

A key feature of the Uncan V8 engine is the ability to apply forces to physics objects based on fields computed on the GPU. This requires an efficient method for transferring the field data from the GPU to the CPU, where the Rapier2D physics simulation is running. Our research indicates that a double-buffering or triple-buffering strategy for GPU readback is the most effective way to minimize latency. This involves using multiple buffers for the readback process. While the CPU is reading from one buffer, the GPU can write to another, thus avoiding stalls in the GPU pipeline.

The `mapAsync` function in WebGPU is the foundation of this asynchronous readback strategy. This function returns a promise that resolves when the GPU data is available to be read by the CPU. This allows the CPU to continue with other tasks while the data is being transferred, which is essential for maintaining a responsive application.

Once the force field data is on the CPU, it can be applied to the Rapier2D rigid bodies. The process involves iterating through each rigid body, determining its position, and then interpolating the force from the GPU-computed field at that position. The interpolated force can then be applied to the rigid body using the `rigidBody.addForce()` function in Rapier2D.

Given that the physics simulation will run at a lower frequency (10Hz) than the rendering (60Hz), force interpolation is crucial. The most recently read force field data can be used for the physics simulation, while for rendering, a smoother visual representation can be achieved by interpolating between the last two force fields.

The research also highlighted the use of Cloud-in-Cell (CIC) deposition with fixed-point encoding for `atomicAdd` operations. This technique, found in the WebGPU-Ocean project, is a valuable reference for accumulating particle data onto a grid. Since WebGPU lacks native atomic operations for floating-point numbers, this method of encoding floats as integers, performing atomic additions, and then decoding the results is a necessary workaround.

### 2.2. Further Research Needed

A deeper investigation into the WebGPU-Ocean project is required to fully understand the implementation of fixed-point atomics for CIC deposition. This will involve a thorough analysis of the project's source code to extract the relevant algorithms and techniques. Additionally, we will search for more examples of applying external forces to Rapier2D rigid bodies to establish best practices and ensure that our implementation is robust and efficient.





## 3. Screened Poisson Implementation with Popov72's FFT

### 3.1. Key Findings

The Uncan V8 engine will feature a sophisticated field solver based on the screened Poisson equation. This equation is a modification of the standard Poisson equation and is particularly well-suited for modeling phenomena such as electric field screening. The equation is given by ∇²φ - κ²φ = αρ, where κ is the screening parameter.

Our research has shown that the most efficient way to solve the screened Poisson equation is to use a spectral method with the Fast Fourier Transform (FFT). This method involves transforming the equation into the frequency domain, where it becomes a simple algebraic equation. The solution is then transformed back to the spatial domain using an inverse FFT.

The Popov72 OceanDemo project provides a high-quality 2D FFT implementation in WebGPU, which can be adapted for our purposes. The implementation is based on the well-known Cooley-Tukey radix-2 FFT algorithm and is highly optimized for performance on the GPU.

The process for solving the screened Poisson equation using this FFT library is as follows:
1.  Compute the FFT of the source term ρ to obtain ρ̂(k).
2.  Solve for the transformed variable φ̂(k) in the frequency domain using the algebraic equation: φ̂(k) = α ρ̂(k) / (|k|² + κ²).
3.  Compute the inverse FFT of φ̂(k) to obtain the final solution φ.

Unlike the pure Poisson equation, the screened Poisson equation does not have a singularity at the DC component (k=0), which simplifies the implementation. The negative sign convention in the equation corresponds to attractive forces, which can be adjusted as needed.

### 3.2. Further Research Needed

A detailed analysis of the WGSL code in the Popov72 project is necessary to fully understand the FFT implementation. This will involve examining the data layout, workgroup size, and shared memory usage to ensure that we can adapt the code correctly for our needs. We will also search for a complete example of solving the screened Poisson equation using a spectral method in WebGPU to further clarify the implementation details and validate our approach.





## 4. Temporal Decoupling with Accumulator Pattern

### 4.1. Key Findings

To ensure a stable and performant simulation, the Uncan V8 engine will employ a temporal decoupling strategy. This involves separating the update rates of the various components of the engine, such as rendering, physics, and AI. This is a standard practice in modern game and simulation engines, and it is essential for achieving a high-quality user experience.

The physics simulation will be updated with a fixed timestep of 10Hz. This is crucial for ensuring the stability and reproducibility of the simulation. Using a variable timestep for physics can lead to a host of problems, including instability, non-determinism, and difficulty in debugging.

In contrast, the rendering will be updated as fast as possible, ideally at 60Hz or higher. This will provide a smooth and responsive visual experience for the user. The rendering rate will naturally vary depending on the performance of the user's hardware.

The accumulator pattern is the chosen technique for implementing this temporal decoupling. This pattern involves accumulating the elapsed time since the last frame. The physics simulation is then updated in fixed-size steps until the accumulator is depleted. This ensures that the physics simulation remains in sync with real-time, regardless of the rendering framerate.

To prevent visual stuttering, the state of the physics objects will be interpolated between physics steps. This will create the illusion of smooth motion, even though the physics simulation is being updated in discrete steps. This is a standard technique that is used in most modern game engines.

### 4.2. Further Research Needed

The research prompt mentions a triple-tier temporal system with rendering at 60Hz, physics at 10Hz, and AI at 0.05Hz. We need to find examples of how to implement this type of system. This will likely involve using multiple accumulators, one for each component. We will also search for production examples of web applications that use this type of temporal decoupling to validate the approach and provide insights into best practices.





## 5. React 19 + Zustand for Real-time Physics

### 5.1. Key Findings

The Uncan V8 engine will be built on a modern web stack, and our research indicates that React 19 and Zustand are an excellent choice for the UI and state management layers. React 19's new compiler, which automatically memoizes components and hooks, will be a major benefit for our real-time physics simulation. This will help to prevent unnecessary re-renders, which can be a major performance bottleneck in complex applications.

Zustand is a lightweight and scalable state management library that is well-suited for real-time applications. Its hook-based API makes it easy to integrate with React, and its efficient state management will help to keep the application running smoothly. The combination of React 19's automatic memoization and Zustand's efficient state management will be a powerful tool for building a high-performance physics engine.

Zustand's state slicing feature will be particularly useful for managing the state of the GPU buffers. We can create a slice of the state that corresponds to a specific buffer and then update it efficiently without affecting other parts of the state. This will be essential for maintaining a high level of performance.

The new `React.useDeferredValue` hook in React 19 will also be useful for deferring the update of non-critical UI elements, such as debug information. This will help to ensure that the main physics simulation and rendering loop are not blocked by non-essential updates.

While React 19's automatic memoization will handle most cases, we may still need to use `React.memo` for performance-critical components, such as the PixiJS container components. By wrapping these components in `React.memo`, we can ensure that they are only re-rendered when their props have changed.

### 5.2. Further Research Needed

We need to find practical examples of using React 19 and Zustand for real-time physics simulations. This will help to clarify the implementation details and provide insights into best practices. We will also search for examples of integrating PixiJS with React 19 and Zustand to understand how to manage the state of the PixiJS canvas and its components.





## 6. 2D FDTD Wave Equation Solver with Sponge Boundaries in WGSL

### 6.1. Key Findings

The Uncan V8 engine will include a 2D FDTD wave equation solver for modeling a variety of physical phenomena. The Finite-Difference Time-Domain (FDTD) method is a robust and widely used technique for solving Maxwell's equations in the time domain. It is a relatively straightforward method to implement and is well-suited for execution on the GPU.

The core of the FDTD method is the discretization of both space and time. The derivatives in the wave equation are then approximated using finite differences. This results in a set of update equations that can be solved iteratively to simulate the propagation of the wave.

To prevent waves from reflecting off the boundaries of the simulation domain, we will use a sponge layer. A sponge layer is a type of absorbing boundary condition that gradually damps the wave as it approaches the boundary, effectively absorbing its energy. This is a standard technique that is used in many FDTD simulations.

The FDTD solver will be implemented as a compute shader in WGSL. The shader will take as input the current state of the wave field and then calculate the state of the field at the next timestep. The sponge layer will be implemented as part of the shader, with the damping factor being applied to the wave field in the sponge region.

The Hacker News thread on the WebGPU-based WiFi simulator provides a valuable real-world example of an FDTD solver implemented in WebGPU. Although the source code is not available, the discussion provides some useful insights into the implementation.

### 6.2. Further Research Needed

We need to find a complete WGSL implementation of a 2D FDTD wave equation solver. The particle life simulation and the WiFi simulator provide some clues, but a complete example would be much more helpful. We also need to find a specific example of how to implement a sponge layer in WGSL. The general concept is clear, but the specific implementation details will be important.





## 7. Persistent Compute Shaders for Fluid Simulation

### 7.1. Key Findings

For the fluid simulation in the Uncan V8 engine, we will need to run compute shaders for extended periods of time. While WebGPU does not have a concept of "persistent" compute shaders in the same way that some other graphics APIs do, it is possible to run compute shaders for long durations. The main challenge is to prevent the GPU from timing out, which can happen if a shader runs for too long without responding.

To avoid a timeout, we will need to poll the device regularly. This can be done by calling `device.poll(wgpu::Maintain::Wait)` in a loop. This will ensure that the GPU remains responsive and that the OS does not reset it.

It is also important to run the compute shader on a separate thread from the main rendering thread. This will prevent the compute shader from blocking the rendering loop and causing the application to become unresponsive.

### 7.2. Further Research Needed

We need to find a complete example of a long-running compute shader in WebGPU. The GitHub discussion provides some clues, but a complete example would be much more helpful. We also need to investigate the performance implications of polling the device regularly. This will help us to find the optimal polling frequency that prevents timeouts without introducing too much overhead.





## 8. Instanced, Animated Meshes for Dense Particle Fields

### 8.1. Key Findings

To render dense particle fields, we will use instanced, animated meshes. This is a standard technique for rendering large numbers of similar objects, and it is well-supported in WebGPU. The basic idea is to create a single mesh for the particle and then use instancing to draw it multiple times, once for each particle.

The animation of the particles will be handled by a compute shader. The compute shader will update the position, rotation, and other properties of each particle. The results of the compute shader will then be passed to the vertex shader, which will use them to transform the vertices of the instanced mesh.

This approach is very efficient, as it minimizes the amount of data that needs to be sent to the GPU. The only data that needs to be sent to the GPU for each frame is the updated particle data. The mesh data is sent to the GPU once and then reused for all of the particles.

### 8.2. Further Research Needed

We need to find a complete example of instanced, animated meshes in WebGPU. The search results provide some general information, but a complete example would be much more helpful. We also need to investigate the performance implications of using instancing for a large number of particles. This will help us to determine the optimal number of instances to use for our application.





## 9. WebGPU Workgroup Optimization for Large-Scale Simulations

### 9.1. Key Findings

Optimizing the use of workgroups in WebGPU is crucial for achieving high performance in large-scale simulations. The size of the workgroup has a significant impact on performance, and it is important to choose a workgroup size that is well-suited to the target hardware. The Medium article on mastering thread calculations in WebGPU provides a good overview of the factors to consider when choosing a workgroup size.

The key takeaway is that the workgroup size should be a multiple of the warp size (or wavefront size) of the target GPU. This ensures that all of the threads in a warp are utilized, which can lead to a significant performance improvement. The article also provides some useful formulas for calculating the optimal workgroup size for a given problem.

### 9.2. Further Research Needed

We need to find more information on the warp sizes of different GPUs. This will help us to choose the optimal workgroup size for a variety of target hardware. We will also search for more examples of workgroup optimization in WebGPU. This will help us to understand how to apply the concepts from the Medium article to our specific application.





## 10. GPU Memory Layout: AoS vs. SoA

### 10.1. Key Findings

The choice of memory layout for GPU data can have a significant impact on performance. The two main options are Array of Structures (AoS) and Structure of Arrays (SoA). In AoS, the data for each object is stored together in a single structure. In SoA, the data for each attribute is stored together in a separate array.

The NVIDIA developer blog post on memory layouts provides a good overview of the trade-offs between AoS and SoA. The key takeaway is that SoA is generally the better choice for GPU programming. This is because it allows for more efficient memory access, as the GPU can read all of the data for a particular attribute in a single memory transaction.

### 10.2. Further Research Needed

We need to find more examples of using SoA in WebGPU. The Reddit thread provides some clues, but a complete example would be much more helpful. We will also search for performance benchmarks that compare the performance of AoS and SoA for a variety of applications. This will help us to quantify the performance benefits of SoA.





## 11. Multi-Layer Rendering with Blend Modes

### 11.1. Key Findings

To create visually rich and complex scenes, the Uncan V8 engine will support multi-layer rendering with a variety of blend modes. This will allow us to composite multiple layers of graphics on top of each other to create a wide range of effects.

The WebGPU fundamentals page on transparency and blending provides a good overview of the different blend modes that are available in WebGPU. The key takeaway is that there are a variety of blend modes to choose from, each with its own unique properties. The most common blend modes are alpha blending, additive blending, and multiplicative blending.

### 11.2. Further Research Needed

We need to find more examples of using blend modes in WebGPU. The WebGPU fundamentals page provides some general information, but more specific examples would be helpful. We will also search for tutorials and articles that provide a more in-depth look at the different blend modes and how they can be used to create different effects.





## 12. Real-time Heatmap Decay on GPU

### 12.1. Key Findings

To visualize the intensity of fields and other data, the Uncan V8 engine will feature a real-time heatmap with a decay effect. This will be implemented as a compute shader that updates the heatmap texture in real-time. The decay effect will be achieved by gradually reducing the intensity of the heatmap over time.

The basic idea is to have a compute shader that takes as input the current heatmap texture and then outputs a new heatmap texture with the decay effect applied. The decay effect can be implemented using a simple formula, such as multiplying the current intensity by a decay factor. The decay factor can be adjusted to control the rate of decay.

### 12.2. Further Research Needed

We need to find a complete example of a real-time heatmap with a decay effect in WebGPU. The search results provide some general information, but a complete example would be much more helpful. We will also search for tutorials and articles that provide a more in-depth look at the different techniques that can be used to create a decay effect.





## 13. WebGPU Device Lost Recovery

### 13.1. Key Findings

Device loss is an unfortunate reality of working with GPUs. It can happen for a variety of reasons, including driver crashes, resource pressure, and long-running shaders. When a device is lost, the `GPUDevice` object and any objects created with it become unusable. This can cause the application to crash or become unresponsive. To prevent this, it is important to handle device loss gracefully.

The Toji.dev article on WebGPU device loss best practices provides a comprehensive overview of how to handle device loss. The key takeaway is that you should always listen for the `lost` promise on the `GPUDevice` object. When this promise resolves, it means that the device has been lost. At this point, you should try to recover from the device loss by creating a new `GPUDevice` and re-creating all of the necessary resources.

The article also provides a number of other best practices, such as always getting a new adapter right before you request a device and providing feedback to the user when a device loss occurs.

### 13.2. Further Research Needed

We need to find a complete example of WebGPU device loss recovery. The Toji.dev article provides some code snippets, but a complete example would be much more helpful. We will also search for more information on how to restore the application state after a device loss. This is a complex topic, and more information would be helpful.





## 14. Progressive Enhancement Strategy

### 14.1. Key Findings

To ensure that the Uncan V8 engine is accessible to as many users as possible, we will use a progressive enhancement strategy. This means that we will start with a baseline of functionality that works on all browsers, and then we will add more advanced features for browsers that support them. The most important aspect of this strategy is to provide a WebGL fallback for browsers that do not support WebGPU.

The Babylon.js forum post on implementing a WebGL fallback for a WebGPU project provides a good overview of the different approaches that can be taken. The key takeaway is that you can use a tool like `tint` to translate your WGSL shaders to GLSL at runtime. This will allow you to use the same shaders for both WebGPU and WebGL.

Another option is to use a node-based material system, such as the one provided by Babylon.js. This will allow you to create your materials once and then have them automatically generate either WGSL or GLSL, depending on whether the user's browser supports WebGPU.

### 14.2. Further Research Needed

We need to find a complete example of a progressive enhancement strategy for WebGPU. The Babylon.js forum post provides some clues, but a complete example would be much more helpful. We will also search for more information on shader translation tools. This will help us to choose the best tool for our needs.





## 15. WebGPU Inspector + PixiJS Integration

### 15.1. Key Findings

Debugging WebGPU applications can be challenging, but there are a number of tools available to help. The WebGPU Inspector is a browser extension that provides a deep level of insight into what is happening on the GPU. It allows you to inspect GPU objects, capture and replay frames, and edit shaders live. This is an invaluable tool for debugging WebGPU applications, and it will be an essential part of our development workflow.

The WebGPU Inspector can be used to debug PixiJS applications that are using the WebGPU renderer. This will allow us to inspect the GPU objects that are being created by PixiJS, as well as the commands that are being sent to the GPU. This will be very helpful for debugging rendering issues and performance problems.

### 15.2. Further Research Needed

We need to find more examples of how to use the WebGPU Inspector to debug PixiJS applications. The search results provide some general information, but more specific examples would be helpful. We will also search for more information on how to use the WebGPU Inspector to profile the performance of PixiJS applications. This will help us to identify and fix performance bottlenecks.





## 16. Fixed-Point Atomic Operations

### 16.1. Key Findings

WebGPU does not have native atomic operations for floating-point numbers. This is a limitation of the underlying hardware, which does not support atomic operations on floating-point values. To perform atomic operations on floating-point numbers, we will need to emulate them using integer atomic operations. This is done by converting the floating-point numbers to a fixed-point representation, performing the atomic operation on the fixed-point numbers, and then converting the result back to a floating-point number.

The Stack Overflow thread on the equivalent of float AtomicAdd in WebGPU provides a good example of how to implement fixed-point atomic operations in WGSL. The example shows how to convert a floating-point number to a fixed-point number, perform an atomic add operation on the fixed-point number, and then convert the result back to a floating-point number.

### 16.2. Further Research Needed

We need to investigate the precision issues that can arise when using fixed-point atomic operations. The precision of the fixed-point representation is limited by the number of bits used to store the number. This can lead to rounding errors, which can accumulate over time. We will also search for information on the performance considerations of fixed-point atomic operations. This will help us to determine whether this is a viable approach for our application.





## 17. f16 Precision for Field Calculations

### 17.1. Key Findings

To optimize the performance of our field calculations, we will investigate the use of the `f16` data type. The `f16` data type is a 16-bit floating-point number that is also known as half-precision floating-point. It is an optional feature in WebGPU, and it can provide significant performance benefits over the `f32` data type. This is because `f16` uses half the memory, which can lead to better cache utilization and reduced memory bandwidth. In addition, some GPUs have specialized hardware for processing `f16` data, which can further improve performance.

The main trade-off with `f16` is that it has a much lower precision than `f32`. This means that it can't represent as many significant digits. For some applications, this loss of precision may not be acceptable. However, for many applications, the performance benefits of `f16` outweigh the loss of precision. For our field calculations, the `f16` data type may be a good choice, as these calculations often do not require a high degree of precision.

### 17.2. Further Research Needed

We need to analyze the precision requirements of our field calculations to determine whether `f16` is a suitable choice. This will involve running some experiments to compare the results of calculations performed with `f16` and `f32`. We will also search for performance benchmarks that compare the performance of `f16` and `f32` for field calculations. This will help us to quantify the performance benefits of `f16`.





## 18. WebGPU Buffer Synchronization

### 18.1. Key Findings

Synchronizing access to buffers in WebGPU is essential for ensuring that all threads have a consistent view of the data. WebGPU does not have a global barrier that can be used to synchronize all threads in a dispatch. Instead, you must use a combination of workgroup barriers and multiple dispatches.

The `workgroupBarrier()` function can be used to synchronize all threads in a workgroup. This is useful for coordinating access to shared memory within a workgroup. The `storageBarrier()` function can be used to synchronize access to storage buffers within a workgroup. This is useful for ensuring that all threads in a workgroup have a consistent view of the data in a storage buffer.

To synchronize all threads in a dispatch, you must use multiple dispatches. This is because there is no way to synchronize threads across workgroups in a single dispatch. To do this, you would first dispatch a compute shader to perform some work, and then you would dispatch a second compute shader to perform some more work. The second compute shader would not start until the first compute shader has finished.

### 18.2. Further Research Needed

We need to find a complete example of how to use multiple dispatches to synchronize all threads in a dispatch. The Stack Overflow thread provides some clues, but a complete example would be much more helpful. We will also search for information on the performance considerations of using multiple dispatches. This will help us to determine whether this is a viable approach for our application.


