# Abstract
I want to create a math visualization tool that can be used to assist in teaching abstract math concepts. At the very least I want to model various groups including but not limited to symmetries of 3D objects, finite abelian groups with their associated lattices, as well as symmetries of permutation groups.

I want to show off the power of using a modern compiled language to create web apps that can also target native devices. This is cutting edge technology that I want to convince my panel of judges to consider teaching in future semesters.


# What Will I Learn
I will learn new ways to visualize mathematical concepts. I will learn how to efficiently render using a lower level graphics API. I will learn how to effectively generate WASM (web-assembly) to optimize applications for the web. I will learn how to make a program cross platform.

# Languages Used
- The project will be mostly written in Rust
- Python for glueing the server and client builds together
- WGSL is the shader language I will be using (WebGPU's shader language)
- Some Javascript for wasm glue

# Libraries Used

- WebGPU
    - Cross platform graphics api
    - Back-end is variable (Web, Vulkan, DX12, Metal, etc)
    - Can target web or native platforms
- Rocket.rs
    - Very flexible library for implementing web servers in Rust
    - I highly recommend Dr. Kowalczyk to look into this (https://rocket.rs/)
- wasm-pack
    - QoL tool used that takes care of compiling to wasm and generating javascript glue
- wasm bind-gen
    - Library for generating wasm bindings
    - Takes care of name mangling so methods can be used by javascript
- Etc
    - Will need some sort of math library
    - Library for printing to browser console
    - Various QoL libraries
    