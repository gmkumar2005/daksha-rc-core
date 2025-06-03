use serde::Serialize;
use utoipa::ToSchema;

pub mod api_routes;
pub mod definition_routes;
pub mod entity_routes;
pub mod health_check;
pub mod user;

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
      "$id": "#/properties/Student",
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
              "$id": "#/properties/fullName",
              "type": "string",
              "title": "Full name"
            },
            "gender": {
              "$id": "#/properties/gender",
              "type": "string",
              "enum": [
                "Male",
                "Female",
                "Other"
              ],
              "title": "Gender"
            },
            "dob": {
              "$id": "#/properties/dob",
              "type": "string",
              "format": "date",
              "title": "DOB"
            },
            "identityHolder": {
              "type": "object",
              "properties": {
                "type": {
                  "$id": "#/properties/type",
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
                  "$id": "#/properties/value",
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
              "$id": "#/properties/email",
              "type": "string",
              "title": "Email"
            },
            "mobile": {
              "$id": "#/properties/mobile",
              "type": "string",
              "title": "Mobile"
            },
            "address": {
              "$id": "#/properties/address",
              "type": "string",
              "title": "Address"
            }
          }
        }
      }
    }
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
