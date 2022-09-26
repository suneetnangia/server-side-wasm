# Overview

This repo provides an accelerator to deploy Web Assemblies (Wasm) for edge workloads, the solution can however be easily adapted to run on any server.

WebAssembly (abbreviated as Wasm) is a binary instruction format for a [stack-based virtual machine](https://andreabergia.com/stack-based-virtual-machines/). It is designed as a portable compilation target for programming languages, enabling deployment on the web for client and server applications, equally. Until recently, Wasm was used mostly in web browsers (client side), this is gradually changing as we see benefits of Wasm being realised on server side or edge as well.

When Wasm is hosted outside of web browser, it needs an interface to interact with the outside world, this interface is provided [WASI (Web Assembly System Interface)](https://wasi.dev/) which is hosted by a standalone runtime called [Wasmtime](https://github.com/bytecodealliance/wasmtime). WASI interfaces makes Wasm portable to multiple operating systems' APIs (e.g. both POSIX and Windows) and [Wasmtime runtime SDKs](https://docs.wasmtime.dev/lang.html) allows us to host the runtime in other applications which may be written in any of the SDK supported languages, enabling the development of Wasm orchestrators.

## Why Wasm

Some of the key tenets/promises of Wasm are defined below:

1. Portable (Hardware, Language and Platform)
2. Secure
3. Near Native Performance
4. Lightweight (when compared to containers)

## Common Challenges & Solutions

Wasm modules run in a sandbox provided by its runtime, this provides the default isolation boundary for the modules i.e. no access to the host resources. This is a great starting point, however most modules require some kind of interfacing with the host or resources outside of its isolation boundary. Interfacing with outside world (from a wasm module perspective) falls into the following categories, usually:

1. Guest wasm module needs to access host file system, this may be required to access a config file or persist data.
2. Guest wasm module needs to make outbound http calls, this may be required to make a call to a RESTful web service.
3. Guest wasm module needs to host a service on socket:port (endpoint), this may be required to host pub-sub messaging server.

This solution/accelerator addresses these common challenges by making use of WASI in 3 primary ways:

1. Using WASI's pre-built APIs e.g. file system APIs to access files and socket APIs to provide a pre-opened socket to the wasm module.
2. Importing funtions from host (described below) to WASI and making them available to wasm modules where they can be called from, as these functions are implemented in the host they have access to resources at the host level. Importing functions also provides a way to achieve IoC/DI (Inversion of Control/Dependency Injection) in wasm module.
3. Exporting functions from wasm modules to WASI and then making them available to host, these functions can be called by the host to initiate the wasm module or other purposes during the execution of the module.

The diagram below provides a conceptual view of the above points:

![alt text](images/wasm-wasi-conceptual.png "Web Assemblies and WASI Conceptual Diagram")

Often on edge or server we have multiple services (microservices), these services have interdependencies to achieve an end goal. To allow this ecosystem of services to work with each other, we need to have a resilient messaging ecosystem e.g. a pub-sub component. In wasm world, this pub-sub component is implemented as wasm module hosting pub-sub messaging server on an endpoint(a WASI pre-opened socket).

![alt text](images/wasm-wasi-edge.png "Web Assemblies on Edge")

The current solution makes use of [Azure IoT Hub](https://docs.microsoft.com/en-gb/azure/iot-hub/) for data plane purposes i.e. sends telemetry to IoT Hub but this is not hardwired, you can send the telemetry to any other RESTful endpoint. This is possible due to access to [Http outbound API](https://deislabs.io/posts/wasi-experimental-http/) from the wasm module.

### WIT Based Host Function Import/Export Approach

[WIT](https://github.com/bytecodealliance/wit-bindgen) (Web-Assembly Interface Types) host functions import approach allows the guest wasm module to import functions from the host app. Consider WIT as a contract definition language between Wasm and host app via WASI, wit-bindgen tool takes this contract and generates bindings for different languages, very similar to how we create bindings for Open API Specs. These import functions are written in host app programming language e.g. Rust/C#/Go, allowing us to write functions to access resources on the host e.g. networking stack to connect to cloud. This is explained [“here (ignore some older termonologies)”](https://radu-matei.com/blog/wasm-components-host-implementations/). Same WIT based approach is used for exporting functions from Wasm module to Host, where they can be called.

![alt text](images/wit-based-design.png "WIT Based Host Function Import Approach")

## Dev Setup

1. Open the solution in Codespaces
2. Compile edge module to wasm32-wasi:
    1. `cd modules`
    2. run `cargo build --target wasm32-wasi`
3. Compile and run app which hosts wasmtime and wasi with imported/exported functions to run wasm edge module:
    1. `cd edge`
    2. run `cargo run`

## References

1. [Lin Clark on WASI @QCon Plus 2021](https://www.infoq.com/presentations/wasi-system-interface/)
2. [SpiderLightning (or, slight)] https://github.com/deislabs/spiderlightning