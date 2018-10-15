## Feature

| Feature          | FlatBuffers | Protobuf | Ssz |
|------------------|-------------|----------|-----|
| Schema           | Yes         | Yes      | No  |
| Zero copy        | Yes         | No       | No  |
| Random access    | Yes         | No       | No  |
| Verifier         | Opt         | Yes      | No  |
| Byte consistency | No          | No       | Yes |


* Schema 方便做版本升级和向后兼容
* Zero copy 避免 memcpy ，允许我们从持久化层获取的数据直接通过网络层传输，但是实践上好像没有办法这样用？
* Random access 实践上我们都是反序列化后再使用，这个特性好像用不到？

## Benchmark
用了两个实际数据结构 Header 和 Block 进行序列化和反序例化测试，其中每个 Block 含有100个 Transaction ，每个 Transaction 含有 3 个 inputs / outputs

### Serialize Header
![serialize_header](.images/serialize_header.svg)

### Serialize Block
![serialize_block](.images/serialize_block.svg)

### Deserialize Header
![deserialize_header](.images/deserialize_header.svg)

### Deserialize Block
![deserialize_block](.images/deserialize_block.svg)

## Data Size
|        | FlatBuffers | Protobuf | Ssz    |
|--------|-------------|----------|--------|
| Header | 352         | 253      | 252    |
| Block  | 268016      | 247356   | 247856 |

FlatBuffers 因为有 vtable 的存在，所以序列化后的数据会比较大一些
