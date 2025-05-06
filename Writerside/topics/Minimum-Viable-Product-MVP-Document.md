# Minimum Viable Product (MVP) Document

<primary-label ref="daksha_rc_core"/>
<secondary-label ref="2024.9"/>

## Overview

Following are the key features of Daksha-RC MVP:

## Daksha-RC MVP #1

<secondary-label ref="wip"/>

### Schema Management

- **Define Schema**: Create and manage schemas that define the structure of registry entries.
  Capture metadata about the schema, such as version, author, and description. **Done**
- **Update Schema**: Modify existing schemas. Manage schema versions. **Done**
- **Validate Schema**: Ensure the schema is a valid document and contains configurations which are allowed by the
  system. **Done**

### Entity Management

- **Create Entity**: Add new record to a registry. **Done**
- **Update Entity**: Modify existing record. **Done**
- **Delete Entity**: Remove record from a registry. **Done**
- **Query Entity**: Retrieve and search through registry entries.
- **Invite Entity**: Invite an entity.
- **Api Documentation**: Provide API documentation for the schema and entity management APIs.

### Policy enforcement

- **Field level privacy**: This functionality enables individuals or organizations to set varying levels of privacy for
  different data fields, depending on how sensitive the information is.
  For instance, less sensitive fields such as a user’s public profile name might have lower privacy restrictions, while
  highly sensitive data like social security numbers would be protected with stricter privacy controls.

  **Example**: A healthcare organization might allow basic demographic information, like a patient’s name or city, to be
  more openly accessible within internal systems, while medical records and personally identifiable information (PII)
  are restricted to only specific authorized personnel.

- **Self registration**: Entities with a self registration policy can be created by the entity itself. It to enforce
  activation workflow for self registered entities.
- **Register via Invitation**: Entities with a invitation policy can be created by the entity itself. It to enforce
  activation workflow for invitation entities.
- **Authentication and Authorization**: Ensure that only authorized users can access the system. Integrate with systems
  like Keycloak, Okta, or Auth0 for authentication and authorization.

## Daksha-RC MVP #2

1. ### Attestation
2. ### Consent Management
3. ### Verification

## Daksha-RC MVP #3

1. ### Credential Management
2. ### Credential Issuance
3. ### Credential Revocation

## Daksha-RC MVP #4

1. ### Digital Wallet
2. ### Consent based sharing of info