# NDK Rest API

## Description

A bridge for interacting with nostr relays using rest API.

## Sample Queries
Get profile of a particular pubkey:
```bash
curl -X POST http://127.0.0.1:7773/events -H 'Content-Type: application/json' -d '[{
    "authors": [
        "c04e8da91853b7fd215102e6aa48477d8e1ba6b3c16902371a153d3784a1b0f7"
    ],
    "kinds": [
        0
    ],
    "limit": 1
}]'
```

Get contact list of a particular pubkey:
```bash
curl -X POST http://127.0.0.1:7773/events -H 'Content-Type: application/json' -d '[{
    "authors": [
        "c04e8da91853b7fd215102e6aa48477d8e1ba6b3c16902371a153d3784a1b0f7"
    ],
    "kinds": [
        3
    ],
    "limit": 1
}]'
```

Publish an event:
```bash
curl -X POST http://127.0.0.1:7773/event -H 'Content-Type: application/json' -d '{
    "content": "hi from alice",
    "created_at": 1678630117,
    "id": "3ee64c68e753a00e77df210fde4b00d9ba7038daf28eff9a49b091c867f70c3d",
    "kind": 1,
    "pubkey": "c04e8da91853b7fd215102e6aa48477d8e1ba6b3c16902371a153d3784a1b0f7",
    "sig": "c8f5808ef7532c2e84c9be2c9a60418f3333638a15a2a5e47e7375cf34ab8c9df30155335415e0b9811fd378baa3630dd4069c2cfa08069edc4034f88dad7baf",
    "tags": []
}'
```

Read list of contacts and their metadata: 
```bash
curl http://127.0.0.1:7773/v1/wss%3A%2F%2Frelay.rip/npub1cp8gm2gc2wml6g23qtn25jz80k8phf4nc95sydc6z57n0p9pkrmsrlh2ad/contacts
```
## License

This project is distributed under the MIT software license - see the [LICENSE](./LICENSE) file for details
