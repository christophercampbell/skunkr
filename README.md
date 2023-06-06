
```shell
cargo run -- --data /path/to/data --port <port> --mdbx
```

```shell
~ » grpcurl -plaintext localhost:7070 list skunkr.Data
skunkr.data.Data.Get
skunkr.data.Data.Scan
skunkr.data.Data.Set
```

```shell
~ » grpcurl -plaintext -d '{"key": "foobar", "value": "dingbat"}' localhost:7070 skunkr.Data.Set
{
  "success": true
}
------------------------------------------------------------------------------------------------------------------------------
~ » grpcurl -plaintext -d '{"key": "foobar"}' localhost:7070 skunkr.Data.Get
{
  "value": "dingbat"
}
```

```shell
~ » grpcurl -plaintext -d '{"key": "mickey", "value": "mouse"}' localhost:7070 skunkr.Data.Set
{
  "success": true
}
------------------------------------------------------------------------------------------------------------------------------
~ » grpcurl -plaintext -d '{"key": "mickey"}' localhost:7070 skunkr.Data.Get
{
  "value": "mouse"
}
```

```shell
~ » grpcurl -plaintext -d '{"from": "foo"}' localhost:7070 skunkr.Data.Scan
{
  "key": "foobar",
  "value": "dingbat"
}
{
  "key": "mickey",
  "value": "mouse"
}
```