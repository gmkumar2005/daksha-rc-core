# RegistryDefinition Design Overview

The **RegistryDefinition** aggregate defines the high-level structure and lifecycle for managing registry definitions.
It follows the **CQRS (Command Query Responsibility Segregation)** pattern, separating state-changing operations (
commands) from state queries (events and read models).

## Purpose

The system manages **definitions that describe business entities and rules** using a JSON schema. These definitions
govern how data is structured and controlled, including aspects like:

- Relationships
- Credentials
- Roles
- Access to private fields

---

## Key Concepts

### Versioning

Each definition has a version to track changes over time. Versions are always positive and increment with each update.

### Definition Identifier

Each definition is uniquely identified using a UUID-based ID.

### Definition Status

A definition moves through various lifecycle stages:

- `None` – Initial state
- `Draft` – Work-in-progress
- `Valid` – Successfully validated
- `Active` – In use
- `Deactivated` – Not in use
- `Invalid` – Validation failed
- `MarkedForDeletion` – Scheduled for removal
- `Modified` – Changed but not yet revalidated

### Events

All changes are recorded as **domain events**, such as:

- Definition created
- Definition updated
- Validation passed or failed
- Definition activated or deactivated
- Definition deleted or loaded from storage

Events include metadata like timestamps and user details.

### Errors

Errors can occur due to:

- Invalid schema data
- Invalid state transitions
- Metadata mismatches (e.g., title)

---

## State Management

The `RegistryDefinition` aggregate maintains:

- A unique identifier
- Current status
- Version number
- Title
- The JSON schema string that defines business rules and structure

---

## State Transitions

State transitions are managed via a **state machine** that enforces:

- Valid paths between statuses
- Business rules for moving between states

---

## Operations

The system supports the following operations:

- **Create** a new definition
- **Update** an existing definition
- **Validate** a definition's schema
- **Activate** or **deactivate** a definition
- **Delete** a definition
- **Load** a definition from storage

Each operation triggers corresponding domain events and follows strict rules.

---

