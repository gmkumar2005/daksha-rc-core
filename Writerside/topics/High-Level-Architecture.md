# High Level Architecture

## Registry (Core)

This core service powers the primary functionalities of Daksha-RC. 
It provides APIs for configuring schemas, managing entities, and handling workflows. 
Additionally, it supports entity management.

## Architecture principles
This core service is designed as a microservice with a [RESTful API](https://en.wikipedia.org/wiki/Representational_state_transfer).
It adopts a [hexagonal architecture](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)), separating the core business logic from the infrastructure.
[CQRS-ES](https://martinfowler.com/bliki/CQRS.html) patterns are used to achieve high scalability and simple maintenance.
[Functional programming](https://en.wikipedia.org/wiki/Functional_programming) principles are applied which allows for high concurrency and low latency.
[Rust programming language](https://www.rust-lang.org/) is used for its efficient memory management performance and safety features.

## Components
### Core Service
The core business logic is implemented as a definitions-manager library.
Business logic is separated from the infrastructure using the repository pattern.
Allows for re-use across different infrastructure implementations. 
for example, initial release will focus on REST API future version will support GraphQL API and GRPC.

## Data Storage
The Command Query Responsibility Segregation (CQRS) and Event Sourcing (ES) design pattern offers flexibility in choosing various data storage solutions. 
This versatility allows developers to select the most appropriate database system based on specific project requirements and performance needs.
### Suitable Database Options

Several types of databases can be effectively utilized with CQRS-ES architecture:

#### Relational Database Management Systems (RDBMS):
- PostgreSQL
- MySQL
- Microsoft SQL Server

#### Document-Oriented Databases:
- MongoDB
- CouchDB
- RavenDB

#### Columnar Databases:
- Apache Cassandra
- Amazon Redshift
- Google Bigtable

### Primary Storage Solution

For the initial release of the system, PostgreSQL has been chosen as the primary storage solution. 
#### This decision is based on several factors:

PostgreSQL supports advanced features like JSONB for flexible schema design, which aligns well with event sourcing principles.
Its open-source nature allows for cost-effective scaling and customization.
It allows for quick rollout in large-scale production environments.

### Testing and Quick Start Solutions

#### To facilitate testing and quick start scenarios like training and demos, SQLite is employed:

SQLite serves as an embedded database, ideal for unit tests and integration tests due to its lightweight nature and zero configuration requirements.
It enables rapid prototyping and development of demo applications or training materials.
SQLite's file-based structure makes it easy to set up isolated test environments and quickly reset database states between tests.

### Considerations for Future Scalability
While PostgreSQL is the primary focus for the initial release, the CQRS-ES architecture allows for future flexibility:
As the system grows, different components can potentially utilize different storage solutions based on their specific needs. 
Event stores might benefit from specialized event sourcing databases like Event Store DB or AxonDB.
Read models could potentially leverage NoSQL databases for improved query performance in certain scenarios.

By adopting CQRS-ES with PostgreSQL as the initial storage solution, 
the system gains flexibility for future growth and optimization while maintaining strong data integrity and performance characteristics. 

## References



