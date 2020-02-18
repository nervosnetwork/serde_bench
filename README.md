## Feature

| Feature          | FlatBuffers | Protobuf | Molecule |
|------------------|-------------|----------|----------|
| Schema           | Yes         | Yes      | Yes      |
| Zero copy        | Yes         | No       | Yes      |
| Random access    | Yes         | No       | Yes      |
| Verifier         | Opt         | Yes      | Opt      |
| Byte consistency | No          | No       | Yes      |


## Benchmark

### Serialize Header
![serialize_header](images/serialize_header.svg)

### Serialize Block
![serialize_block](images/serialize_block.svg)

### Deserialize Header
![deserialize_header](images/deserialize_header.svg)

### Deserialize Block
![deserialize_block](images/deserialize_block.svg)

## Data Size
|        | FlatBuffers | Protobuf | Molecule |
|--------|-------------|----------|----------|
| Header | 352         | 253      | 304      |
| Block  | 268016      | 247356   | 267120   |
