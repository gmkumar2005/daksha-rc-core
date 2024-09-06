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



