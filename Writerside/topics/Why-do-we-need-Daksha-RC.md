# Core Capabilities

## Registry

A governing body or authority can establish a registry that serves as the single source of truth.
    
### Specify the schema according to the policy.
#### Define fields and their types: 
You can create new fields in the registry to store specific information and specify the type of data for each field. 
For example, in a student registry, fields like student name, father's name, mother's name, school name, and age can be defined, with the age field restricted to numeric values.    
#### Field level privacy: 
This feature enables individuals or organizations to set varying privacy levels for different fields based on the data's sensitivity. 
For instance, in a patient registry, adopters might restrict access to fields with patient health information to authorized healthcare professionals, while fields with less sensitive data, like patient demographics, could be accessible to a broader range of users.    
#### Consent framework at schema level
### Create entities:
- Bulk invite/onboard users via CSV
- Link to external systems
- Self-register
- Decide visibility of who can view the content at the field level
### Define ownership
- Define which entities can log in and how authorization is handled
### Discovery: 
- Control the visibility of discovery attributes through access control mechanisms such as consent-based sharing. 
- Users can define which discovery attributes are:
  - Public (visible to all users)
  - Private (visible only to the user)
  - Consent-based (visible to selected users or groups based on their consent)
### Analytics: 
- Basic analytics for registries

## Verifiable Credentials (VC)

### Define Attestation and Claim Workflow:
- Tenants (users) can define an 'add-on' workflow, allowing them to customize the workflow steps to meet their needs. 
- They can also configure claim approval to be either automatic or manual.    The system used for attestation can be an internal system like Sunbird RC or an external system that the tenant connects to.
- Multi-level attestation allows a claim to be reviewed by multiple parties or approval levels.
- Define Validity

### Consent:

#### Grant or revoke consent: 
- Capable of handling requests to grant or revoke access permissions.
#### Consent Auditing
- Issuance services
- W3C compliant
- Create and Issue Verifiable Credentials
- Update and Revocation
- Unique ID generation

#### VC Verification
- Offline verification of Verifiable Credentials
- Consent based access: Allows users to grant consent to another system or person to access Verifiable Credentials for transactions or interactions.