# RegistryResource Design Overview

## Purpose

The `RegistryResource` represents the core domain aggregate for managing registry entities.
It is responsible for maintaining the current state of an entity and enforcing domain rules when changes are proposed.
Through this abstraction, the system ensures that all modifications—whether creation, updates, or deletions—are
validated, versioned, and recorded consistently.
This provides a single source of truth for an entity's lifecycle and supports robust auditability and traceability.

It handles entities in a registry through well-defined commands, events, and aggregates.

---

## Commands

Commands initiate all state changes and are categorized by their intent:

- **Create**: Introduce a new entity.
- **Modify**: Update an existing entity.
- **Delete**: Remove or deactivate an entity.

Each command must be validated and must comply with domain rules and schema definitions.

---

## Operations

### 1. **Create Entity**

- Adds a new registry entity.
- Requires validation against the entity schema.
- Transitions entity from `None` to `Active`.

### 2. **Modify Entity**

- Updates data of an existing entity.
- Allowed only in valid states (e.g., `Active`, `Modified`).
- Update the schema version.

### 3. **Delete Entity**

- Marks the entity for removal (soft delete).
- Transition to `MarkedForDeletion`.

### 4. **Deactivate Entity**

- Prevents further use without removing the entity.
- Transition to `Deactivated`.

### 5. **Invite Entity**

- Used for pre-activation workflows.
- Transition to `Invited`.

---

## Events

The file defines the following events related to entities:

* `EntityCreated`: This event signifies that a new entity has been successfully created. It includes details such as the
  entity's ID, the ID and version of the registry definition it conforms to, the entity's data, its type, creation
  timestamp, and the user who created it.
* `EntityInvited`: This event indicates that an entity has been invited, although the specific meaning of "invited" is
  not clear from the provided code. The event data mirrors `EntityCreated`.
* `EntityUpdated`: This event signifies that an existing entity has been modified. It includes similar details to
  `EntityCreated`, along with the modification timestamp, the user who modified it, and the new version number of the
  entity.
* `EntityDeleted`: This event signifies that an entity has been marked for deletion. It includes the ID of the deleted
  entity, time of deletion and who deleted it.

## Entity Statuses

Entities can be in one of several lifecycle states:

- **None**: Not yet created.
- **Active**: Fully available for use.
- **Invited**: In pre-activation state.
- **Modified**: Has been updated after creation.
- **Deactivated**: Disabled but retained.
- **MarkedForDeletion**: Scheduled for removal.

---


