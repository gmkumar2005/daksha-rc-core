## Definitions manger 

### Requirements
The definitions manager should be able to load definitions from a file 
One entity from one file. For eg : Institute.json should create entity with name Institute
Entity contains definitions
Definition contains SubSchemas and OSconfigurations 
Definition is created by reading all the contents of the file
Definitions-Map contains Title and Definitions

SchemaManager manages SchemaDoc aka SchemaDocument. 
It has the following attributes
1. Id
2. Title - Initially we will support one entity per schema
3. Version
4. Schema - The schema of the document as a string
5. Status - Active or Inactive or Invalid 

### Analysis
- One SchemaDocument has one Definition and OSConfigurations
#### Potential Events
- Definition can be loaded - DefLoaded
- Definition can be created - DefCreated
- Definition can be updated - DefUpdated
  - Add properties - PropertiesAdded
  - Remove properties - PropertiesRemoved
  - Replace properties - PropertiesReplaced
  - Modify Visibility - VisibilityModified
  - Add AttestationPolicies - AttestationPoliciesAdded
  - Replace AttestationPolicies - AttestationPoliciesReplaced
  - Add OwnerShipAttributes - OwnerShipAttributesAdded
  - Replace OwnerShipAttributes - OwnerShipAttributesReplaced
- Definition can be deleted - DefDeleted
- Definition can be validated - DefValidated
- Definition can be activated - DefActivated
- Definition can be deactivated - DefDeactivated

Definition can have the following states
- Draft
- Valid
- Active
- Deactivated
- Invalid
- MarkedForDeletion

Event Streams
1 - DefinitionEvents

### Aggregate Design
#### Aggregate Root is Definition 
##### Events 
#[stream(DefStateEvents, [DefLoaded, DefCreated, DefUpdated, DefDeleted, DefValidated, DefActivated, DefDeactivated])]
#[stream(DefChangeEvents, [PropertiesAdded, PropertiesRemoved, PropertiesReplaced, VisibilityModified, AttestationPoliciesAdded, AttestationPoliciesReplaced, OwnerShipAttributesAdded, OwnerShipAttributesReplaced])]

##### Errors 
InvalidJson
InvalidSchema
InvalidDefinition
DefinitionAlreadyExists
DefinitionNotFound
DefinitionNotValid
DefinitionNotActive

##### Mutations
DefState 

##### Commands (Commands perform the decision-making)
LoadDefinition
CreateDefinition
UpdateDefinition
DeleteDefinition
ValidateDefinition
ActivateDefinition
DeactivateDefinition

```








