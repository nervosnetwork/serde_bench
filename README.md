## Feature

| Feature           | FlatBuffers | Protobuf | Molecule |
|-------------------|-------------|----------|----------|
| Schema            | Yes         | Yes      | Yes      |
| Zero copy         | Yes         | No       | Yes      |
| Random access*    | Yes         | No       | Yes      |
| Verifier          | Opt         | Yes      | Opt      |
| Byte consistency* | No          | No       | Yes      |


Random access: You can read just one field of a message without parsing the whole thing.

Byte consistency: The same data can be represented in multiple different byte sequences or not, although usually FlatBuffers or Protobuf serializer will produce the same output from the same data, it is not a formal guarantee, so you can't just glance at two outputs (or their hashes) and conclude "if the bytes are the same the data is the same, otherwise the data is different".

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
