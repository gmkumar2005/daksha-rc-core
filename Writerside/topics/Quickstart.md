# Developer Guide: Getting Started with daksha-rc-core

## Overview

Welcome! This guide will walk you through checking out, building, and running the **daksha-rc-core** project.
This project is written in Rust and utilizes several modern libraries for performance and flexibility.

## Before you start

Before you begin, ensure you have the following installed:

- **Rust toolchain** (recommended: version 1.86.0 or later)
    - [Install Rust](https://www.rust-lang.org/tools/install)

- **Git** for version control
    - [Install Git](https://git-scm.com/downloads)

- **Docker**

## Clone the Repository

```shell
git clone https: //github.com/Daksha-RC/daksha-rc-core.git
cd daksha-rc-core

```

## Build the Project

The project uses Cargo, Rust's build tool and package manager.

```shell
cargo build 
```

## Test

```shell
cargo test
```

## NexTest (faster testing)

```shell
cargo nextest run
```

## Run the Project

```shell
cargo shuttle run

```

## Access the api

- The REST APIs will be accessible at [http://localhost:8000](http://localhost:8000)
- The Scalar API is available at [http://localhost:8000/scalar](http://localhost:8000/scalar)
- The Swagger UI is available at [http://localhost:8000/swagger-ui/](http://localhost:8000/swagger-ui/)
