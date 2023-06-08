Simple KV store with an iterating scan interface, IO bound to limits of storage. No (yet) authentication or encryption. 

```shell
cargo run -- --data /path/to/data --port <port> --mdbx
```

```shell
~ » grpcurl -plaintext localhost:7070 list skunkr.Data
skunkr.data.Data.Get
skunkr.data.Data.Scan
skunkr.data.Data.Set
```

Keys and data are base64 encoded byte arrays.

```shell
echo foobar | base64
Zm9vYmFyCg==

echo dingbat | base64
ZGluZ2JhdAo=
```

```shell
~ » grpcurl -plaintext -d '{"key": "Zm9vYmFyCg==", "value": "ZGluZ2JhdAo="}' localhost:7070 skunkr.Data.Set
{
  "success": true
}
------------------------------------------------------------------------------------------------------------------------------
~ » grpcurl -plaintext -d '{"key": "Zm9vYmFyCg=="}' localhost:7070 skunkr.Data.Get
{
  "value": "ZGluZ2JhdAo="
}
```

```shell
echo mickey | base64
bWlja2V5Cg==

echo mouse | base64
bW91c2UK
```

```shell
~ » grpcurl -plaintext -d '{"key": "bWlja2V5Cg==", "value": "bW91c2UK"}' localhost:7070 skunkr.Data.Set
{
  "success": true
}
------------------------------------------------------------------------------------------------------------------------------
~ » grpcurl -plaintext -d '{"key": "bWlja2V5Cg=="}' localhost:7070 skunkr.Data.Get
{
  "value": "bW91c2UK"
}
```

```
echo foo | base64
Zm9vCg==
```

```shell
~ » grpcurl -plaintext -d '{"from": "Zm9vCg=="}' localhost:7070 skunkr.Data.Scan
{
  "key": "Zm9vYmFyCg==",
  "value": "ZGluZ2JhdAo="
}
{
  "key": "bWlja2V5Cg==",
  "value": "bW91c2UK"
}
```