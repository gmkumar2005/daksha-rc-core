use serde::Serialize;
use utoipa::ToSchema;

pub mod api_routes;
pub mod definition_routes;
pub mod entity_routes;
pub mod health_check;
pub mod user;

pub const INSURANCE_EXAMPLE: &str = r###"{
  "$schema": "http://json-schema.org/draft-07/schema",
  "type": "object",
  "properties": {
    "Insurance": {
      "$ref": "#/definitions/Insurance"
    }
  },
  "required": [
    "Insurance"
  ],
  "title":"Insurance",
  "definitions": {
    "Insurance": {
      "$id": "#/properties/Insurance",
      "type": "object",
      "title": "Insurance",
      "required": [
        "policyNumber",
        "policyName",
        "policyExpiresOn",
        "policyIssuedOn",
        "fullName",
        "dob"
      ],
      "properties": {
        "policyNumber": {
          "type": "string"
        },
        "policyName": {
          "type": "string"
        },
        "policyExpiresOn": {
          "type": "string",
          "format": "date-time"
        },
        "policyIssuedOn": {
          "type": "string",
          "format": "date-time"
        },
        "benefits": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "fullName": {
          "type": "object",
          "properties": {
            "firstName": {
            "type": "string",
            "format": "firstName"
            }
          },
          "title": "Full Name"
        },
        "dob": {
          "type": "string",
          "format": "date"
        },
        "gender": {
          "type": "string",
          "enum": [
            "Male",
            "Female",
            "Other"
          ]
        },
        "mobile": {
          "type": "string",
          "title": "Mobile number"
        },
        "email": {
          "type": "string",
          "title": "Email ID"
        }
      }
    }
  },
  "_osConfig": {
    "credentialTemplate": {
      "@context": [
        "https://www.w3.org/2018/credentials/v1",
        {
          "@context": {
            "@version": 1.1,
            "@protected": true,
            "id": "@id",
            "type": "@type",
            "schema": "https://schema.org/",
            "InsuranceCredential": {
              "@id": "did:InsuranceCredential",
              "@context": {
                "@version": 1.1,
                "@protected": true,
                "id": "@id",
                "type": "@type",
                "dob": "schema:birthDate",
                "email": "schema:email",
                "gender": "schema:gender",
                "mobile": "schema:telephone",
                "benefits": "schema:benefits",
                "fullName": "schema:name",
                "policyName": "schema:Text",
                "policyNumber": "schema:Text"
              }
            }
          }
        },
        {
          "HealthInsuranceCredential": {
            "@id": "InsuranceCredential"
          },
          "LifeInsuranceCredential": {
            "@id": "HealthInsuranceCredential"
          }
        }
      ],
      "type": [
        "VerifiableCredential",
        "LifeInsuranceCredential"
      ],
      "issuer": "Registry",
      "issuanceDate": "{{policyIssuedOn}}",
      "expirationDate": "{{policyExpiresOn}}",
      "credentialSubject": {
        "id": "did:{{osid}}",
        "dob": "{{dob}}",
        "type": "InsuranceCredential",
        "email": "{{email}}",
        "gender": "{{gender}}",
        "mobile": "{{mobile}}",
        "benefits": "{{benefits}}",
        "fullName": "{{fullName}}",
        "policyName": "{{policyName}}",
        "policyNumber": "{{policyNumber}}"
      }
    },
    "certificateTemplates": {
      "first": "minio://Insurance/1-68619c95-3f40-45b8-b6ba-56eba055dc11/email/documents/3165a481-8078-447c-8cc0-f310869cb40d-Insurancetemplate.html"
    },
    "osComment": [],
    "privateFields": [],
    "systemFields": [
      "_osSignedData",
      "_osCredentialId",
      "_osAttestedData"
    ],
    "indexFields": [],
    "uniqueIndexFields": [],
    "roles": ["Official"],
    "inviteRoles": ["Official"],
    "attestationPolicies": [
      {
        "name": "cropApprovalPolicy",
        "attestationProperties": {
          "policyExpiresOn": "$.policyExpiresOn",
          "policyNumber": "$.policyNumber",
          "policyName": "$.policyNumber",
          "fullName": "$.fullName"
        },
        "type": "MANUAL",
        "attestorPlugin": "did:internal:ClaimPluginActor?entity=Official",
        "conditions": "(ATTESTOR#$.Gender#.equalsIgnoreCase('male'))",
        "credentialTemplate": {
          "@context": [
            "https://www.w3.org/2018/credentials/v1",
            {
              "@context": {
                "@version": 1.1,
                "@protected": true,
                "id": "@id",
                "type": "@type",
                "schema": "https://schema.org/",
                "InsuranceCredential": {
                  "@id": "did:InsuranceCredential",
                  "@context": {
                    "@version": 1.1,
                    "@protected": true,
                    "id": "@id",
                    "type": "@type",
                    "policyExpiresOn": "schema:expires",
                    "policyName": "schema:Text",
                    "policyNumber": "schema:Text"
                  }
                }
              }
            }
          ],
          "type": [
            "VerifiableCredential",
            "InsuranceCredential"
          ],
          "issuer": "Registry",
          "expirationDate": "{{policyExpiresOn}}",
          "credentialSubject": {
            "id": "did:{{policyName}}:{{policyNumber}}",
            "type": "InsuranceCredential",
            "policyName": "{{policyName}}",
            "policyNumber": "{{policyNumber}}",
            "policyExpiresOn": "{{policyExpiresOn}}"
          }
        }
      }
    ],
    "ownershipAttributes": [
      {
        "userId": "$.email",
        "email": "$.email",
        "mobile": "$.mobile"
      }
    ]
  }
}"###;

pub const INSURANCE_OFFICIAL_EXAMPLE: &str = r###"{
  "$schema": "http://json-schema.org/draft-07/schema",
  "type": "object",
  "properties": {
    "Official": {
      "$ref": "#/definitions/Official"
    }
  },
  "required": [
    "Official"
  ],
  "title": "Official",
  "definitions": {
    "Official": {
      "$id": "#/properties/Official",
      "type": "object",
      "title": "The Official Schema",
      "required": [
        "Name",
        "Phone",
        "email",
        "State",
        "Category"
      ],
      "properties": {
        "Name": {
          "type": "string"
        },
        "Gender": {
          "type": "string"
        },
        "Phone": {
          "type": "string"
        },
        "email": {
          "type": "string"
        },
        "State": {
          "type": "string"
        },
        "Category": {
          "type": "string"
        },
        "Designation": {
          "type": "string"
        },
        "Department": {
          "type": "string"
        }
      }
    }
  },
  "_osConfig": {
    "systemFields": [
      "osCreatedAt",
      "osUpdatedAt",
      "osCreatedBy",
      "osUpdatedBy"
    ],
    "roles": ["admin"],
    "inviteRoles": ["admin"],
    "ownershipAttributes": [
      {
        "email": "/email",
        "mobile": "/Phone",
        "userId": "/Phone"
      }
    ]
  }
}
"###;

pub const STUDENT_EXAMPLE: &str = r###"{
  "$schema": "http://json-schema.org/draft-07/schema",
  "type": "object",
  "properties": {
    "Student": {
      "$ref": "#/definitions/Student"
    }
  },
  "required": [
    "Student"
  ],
  "title": "Student",
  "definitions": {
    "Student": {
      "type": "object",
      "title": "The Student Schema",
      "required": [],
      "properties": {
        "identityDetails": {
          "type": "object",
          "title": "Identity Details",
          "description": "Identity Details",
          "required": [],
          "properties": {
            "fullName": {
              "type": "string",
              "title": "Full name"
            },
            "gender": {
              "type": "string",
              "enum": [
                "Male",
                "Female",
                "Other"
              ],
              "title": "Gender"
            },
            "dob": {
              "type": "string",
              "format": "date",
              "title": "DOB"
            },
            "identityHolder": {
              "type": "object",
              "properties": {
                "type": {
                  "type": "string",
                  "$comment": "Nationality",
                  "title": "ID Type",
                  "enum": [
                    "AADHAR",
                    "PAN",
                    "LICENSE",
                    "OTHER"
                  ]
                },
                "value": {
                  "type": "string",
                  "$comment": "Nationality",
                  "title": "ID Value"
                }
              }
            }
          }
        },
        "contactDetails": {
          "type": "object",
          "title": "Contact Details",
          "description": "Contact Details",
          "required": [],
          "properties": {
            "email": {
              "type": "string",
              "title": "Email"
            },
            "mobile": {
              "type": "string",
              "title": "Mobile"
            },
            "address": {
              "type": "string",
              "title": "Address"
            }
          }
        }
      }
    }
  },
  "_osConfig": {
    "osComment": [
      "This section contains the OpenSABER specific configuration information",
      "privateFields: Optional; list of field names to be encrypted and stored in database",
      "signedFields: Optional; list of field names that must be pre-signed",
      "indexFields: Optional; list of field names used for creating index. Enclose within braces to indicate it is a composite index. In this definition, (serialNum, studentCode) is a composite index and studentName is a single column index.",
      "uniqueIndexFields: Optional; list of field names used for creating unique index. Field names must be different from index field name",
      "systemFields: Optional; list of fields names used for system standard information like created, updated timestamps and userid"
    ],
    "privateFields": [
      "$.identityDetails.dob",
      "$.identityDetails.identityType",
      "$.identityDetails.identityValue"
    ],
    "internalFields": [
      "$.contactDetails.email",
      "$.contactDetails.mobile",
      "$.contactDetails.address"
    ],
    "signedFields": [],
    "indexFields": [
      "studentName"
    ],
    "uniqueIndexFields": [
      "identityValue"
    ],
    "systemFields": [
      "_osCreatedAt",
      "_osUpdatedAt",
      "_osCreatedBy",
      "_osUpdatedBy",
      "_osAttestedData",
      "_osClaimId",
      "_osState"
    ],
    "attestationAttributes": [
      "educationDetails",
      "nationalIdentifier"
    ],
    "attestationPolicies": [
      {
        "name": "attestationEducationDetails",
        "properties": [
          "educationDetails/[]"
        ],
        "paths": [
          "$.educationDetails[?(@.osid == 'PROPERTY_ID')]['instituteName', 'program', 'graduationYear', 'marks']",
          "$.identityDetails['fullName']"
        ],
        "type": "MANUAL",
        "attestorEntity": "Teacher",
        "attestorPlugin": "did:internal:Claim?entity=Teacher",
        "conditions": "(ATTESTOR#$.experience.[*].instituteOSID#.contains(REQUESTER#$.instituteOSID#) && ATTESTOR#$.experience[?(@.instituteOSID == REQUESTER#$.instituteOSID#)]['_osState']#.contains('PUBLISHED'))"
      }
    ],
    "autoAttestationPolicies": [
      {
        "parentProperty": "identityDetails",
        "property": "identityHolder",
        "nodeRef": "$.identityDetails.identityHolder",
        "valuePath": "$.identityDetails.identityHolder.value",
        "typePath": "$.identityDetails.identityHolder.type"
      }
    ],
    "subjectJsonPath": "mobile",
    "ownershipAttributes": [
      {
        "email": "/contactDetails/email",
        "mobile": "/contactDetails/mobile",
        "userId": "/contactDetails/mobile"
      }
    ],
    "inviteRoles": [
      "anonymous"
    ],
    "roles": [
      "anonymous"
    ]
  }
}"###;
pub const TEACHER_EXAMPLE: &str = r###"{
  "$schema": "http://json-schema.org/draft-07/schema",
  "type": "object",
  "properties": {
    "Teacher": {
      "$ref": "#/definitions/Teacher"
    }
  },
  "required": [
    "Teacher"
  ],
  "title": "Teacher",
  "definitions": {
    "Teacher": {
      "$id": "#/properties/Teacher",
      "type": "object",
      "title": "The Teacher Schema",
      "required": [
      ],
      "properties": {
        "personalDetails": {
          "type": "object",
          "properties": {
            "email": {
              "type": "string"
            }
          }
        },
        "identityDetails": {
          "type": "object",
          "properties": {
            "id": {
              "type": "string"
            },
            "value": {
              "type": "string"
            }
          }
        }
      }
    }
  }
}"###;

pub const CONSULTANT_EXAMPLE: &str = r###"{
  "$schema": "http://json-schema.org/draft-07/schema",
  "title": "Consultant",
  "type": "object",
  "properties": {
    "name": {
      "type": "string",
      "description": "Full name of the consultant"
    },
    "expertise": {
      "type": "array",
      "items": {
        "type": "string"
      },
      "description": "Areas of specialization or domain expertise"
    },
    "certifications": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/Credential"
      },
      "description": "Verified professional certifications"
    },
    "experienceYears": {
      "type": "integer",
      "description": "Total years of experience"
    },
    "portfolio": {
      "type": "array",
      "items": {
        "type": "string",
        "format": "uri"
      },
      "description": "Links to past work, case studies, or testimonials"
    },
    "availability": {
      "type": "string",
      "enum": [
        "Available",
        "Unavailable",
        "Limited"
      ],
      "description": "Current availability status for new engagements"
    },
    "location": {
      "$ref": "#/definitions/Location"
    },
    "contactInformation": {
      "$ref": "#/definitions/Contact"
    }
  },
  "required": [
    "name",
    "expertise",
    "contactInformation"
  ],
  "definitions": {
    "Location": {
      "type": "object",
      "properties": {
        "city": {
          "type": "string"
        },
        "state": {
          "type": "string"
        },
        "country": {
          "type": "string"
        }
      },
      "required": [
        "city",
        "country"
      ]
    },
    "Contact": {
      "type": "object",
      "properties": {
        "email": {
          "type": "string",
          "format": "email"
        },
        "phone": {
          "type": "string"
        }
      },
      "required": [
        "email"
      ]
    },
    "Credential": {
      "type": "object",
      "properties": {
        "credentialId": {
          "type": "string"
        },
        "issuer": {
          "type": "string"
        },
        "issueDate": {
          "type": "string",
          "format": "date"
        }
      },
      "required": [
        "credentialId",
        "issuer"
      ]
    }
  }
}
"###;
pub const CLIENT_EXAMPLE: &str = r###"{
  "$schema": "http://json-schema.org/draft-07/schema",
  "title": "Client",
  "type": "object",
  "properties": {
    "name": {
      "type": "string",
      "description": "Full name of the client"
    },
    "organization": {
      "type": "string",
      "description": "Associated business or entity"
    },
    "industry": {
      "type": "string",
      "description": "Industry sector of the client"
    },
    "requirements": {
      "type": "string",
      "description": "Project needs or consultation requirements"
    },
    "location": {
      "$ref": "#/definitions/Location"
    },
    "contactInformation": {
      "$ref": "#/definitions/Contact"
    },
    "verifiedCredentials": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/Credential"
      }
    }
  },
  "required": [
    "name",
    "organization",
    "requirements"
  ],
  "definitions": {
    "Location": {
      "type": "object",
      "properties": {
        "city": {
          "type": "string"
        },
        "state": {
          "type": "string"
        },
        "country": {
          "type": "string"
        }
      },
      "required": [
        "city",
        "country"
      ]
    },
    "Contact": {
      "type": "object",
      "properties": {
        "email": {
          "type": "string",
          "format": "email"
        },
        "phone": {
          "type": "string"
        }
      },
      "required": [
        "email"
      ]
    },
    "Credential": {
      "type": "object",
      "properties": {
        "credentialId": {
          "type": "string"
        },
        "issuer": {
          "type": "string"
        },
        "issueDate": {
          "type": "string",
          "format": "date"
        }
      },
      "required": [
        "credentialId",
        "issuer"
      ]
    }
  }
}
"###;

pub const TEACHER_SMITH_EXAMPLE: &str = r###"{
  "Teacher": {
    "personalDetails": {
      "email": "teacher.smith@school.edu"
    },
    "identityDetails": {
      "id": "TCHR-2023-001",
      "value": "123456789"
    }
  }
}"###;

pub const STUDENT_JOHN_EXAMPLE: &str = r###"{
  "Student": {
    "identityDetails": {
      "fullName": "John Smith",
      "gender": "Male",
      "dob": "2000-05-15",
      "identityHolder": {
        "type": "AADHAR",
        "value": "1234-5678-9012"
      }
    },
    "contactDetails": {
      "email": "john.smith@example.com",
      "mobile": "+91-9876543210",
      "address": "123 Education Street, Knowledge City, Learning State - 100001"
    }
  }
}"###;

pub const CONSULTANT_SARAH_EXAMPLE: &str = r###"{
  "name": "Dr. Sarah Johnson",
  "expertise": [
    "Cloud Architecture",
    "System Design",
    "DevOps",
    "Microservices"
  ],
  "certifications": [
    {
      "credentialId": "AWS-SAP-123456",
      "issuer": "Amazon Web Services",
      "issueDate": "2023-06-15"
    },
    {
      "credentialId": "GCP-APD-789012",
      "issuer": "Google Cloud",
      "issueDate": "2023-03-20"
    }
  ],
  "experienceYears": 12,
  "portfolio": [
    "https://github.com/sarahjohnson",
    "https://linkedin.com/in/sarahjohnson",
    "https://consulting-cases.example.com/cloud-migration-study"
  ],
  "availability": "Limited",
  "location": {
    "city": "Seattle",
    "state": "Washington",
    "country": "USA"
  },
  "contactInformation": {
    "email": "sarah.johnson@consulting.example.com",
    "phone": "+1-206-555-0123"
  }
}"###;

pub const CLIENT_JOHN_EXAMPLE: &str = r###"{
  "name": "John Smith",
  "organization": "Tech Innovations Ltd",
  "industry": "Software Development",
  "requirements": "Need consultation for cloud migration and microservices architecture",
  "location": {
    "city": "San Francisco",
    "state": "California",
    "country": "USA"
  },
  "contactInformation": {
    "email": "john.smith@techinnovations.com",
    "phone": "+1-555-123-4567"
  },
  "verifiedCredentials": [
    {
      "credentialId": "CERT-2023-001",
      "issuer": "ISO 27001",
      "issueDate": "2023-01-15"
    },
    {
      "credentialId": "CERT-2023-002",
      "issuer": "CMMI Level 5",
      "issueDate": "2023-03-20"
    }
  ]
}"###;

// SchemaDef created with id: fa0f1791-8ddc-5934-fc00-2aff27a84ddf for title: Client
// SchemaDef created with id: 8568a0b3-8900-e4d2-6e72-e3ad5792288e for title: Consultant
// domain error: Definition Already Exists for : Student with id: 1bd23c91-3379-b65b-11cc-64984050e35c
// SchemaDef created with id: e757aa6e-d39a-2db7-6345-473ddd8aadb2 for title: Teacher

/// Example fa0f1791-8ddc-5934-fc00-2aff27a84ddf for title: Client
/// 8568a0b3-8900-e4d2-6e72-e3ad5792288e for title: Consultant
/// 1bd23c91-3379-b65b-11cc-64984050e35c for title: Student
/// e757aa6e-d39a-2db7-6345-473ddd8aadb2 for title: Teacher
///
/// Standard error response structure
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error code or identifier (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Detailed human-readable description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    /// Main error message
    pub message: String,
}
