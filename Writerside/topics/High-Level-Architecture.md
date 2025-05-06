# High Level Architecture

## Registry (Core)

This core service powers the primary functionalities of Daksha-RC.
It provides APIs for configuring schemas, managing entities, and handling workflows.
Additionally, it supports entity management.

## Architecture principles

This core service is designed as a microservice with
a [RESTful API](https://en.wikipedia.org/wiki/Representational_state_transfer).
It adopts a [hexagonal architecture](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)), separating the
core business logic from the infrastructure.
[CQRS-ES](https://martinfowler.com/bliki/CQRS.html) patterns are used to achieve high scalability and simple
maintenance.
[Functional programming](https://en.wikipedia.org/wiki/Functional_programming) principles are applied which allows for
high concurrency and low latency.
[Rust programming language](https://www.rust-lang.org/) is used for its efficient memory management performance and
safety features.

## Components

### Core Service

The core business logic is implemented as a definitions-manager library.
Business logic is separated from the infrastructure using the repository pattern.
Allows for re-use across different infrastructure implementations.
for example, initial release will focus on REST API future version will support GraphQL API and GRPC.

## Data Storage

The Command Query Responsibility Segregation (CQRS) and Event Sourcing (ES) design pattern offers flexibility in
choosing various data storage solutions.
This versatility allows developers to select the most appropriate database system based on specific project requirements
and performance needs.
The write side is built to support postgres database.
The projections aka read-side can be extended to support multiple storages.

## Suitable Database Options

Several types of databases can be effectively utilized with CQRS-ES architecture:

### Primary Storage Solution

For the initial release of the system, PostgreSQL has been chosen as the primary storage solution.

#### This decision is based on several factors:

PostgreSQL supports advanced features like JSONB for flexible schema design, which aligns well with event sourcing
principles.
Its open-source nature allows for cost-effective scaling and customization.
It allows for quick rollout in large-scale production environments.

### Considerations for Future Scalability

While PostgreSQL is the primary focus for the initial release, the CQRS-ES architecture allows for future flexibility:
As the system grows, different components can potentially utilize different storage solutions based on their specific
needs.

By adopting CQRS-ES with PostgreSQL as the initial storage solution,
the system gains flexibility for future growth and optimization while maintaining strong data integrity and performance
characteristics.




