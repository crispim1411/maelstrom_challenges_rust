# Maelstrom ChaUllenges
Solução dos exercícios de Sistemas Distribuidos disponíveis em [fly.io/dist-sys](https://fly.io/dist-sys/)

#### Estrutura das mensagens 
```rust
Message<T> {
    src: String,
    dst: String,
    body: {
        msg_id: Option<usize>,
        in_reply_to: Option<usize>,
        payload: T,
    }
}
```

#### Estrutura dos eventos
```rust
Event {
    Message<Payload>,
    Gossip,
    EOF,
}
```

## #1 Echo
Implementar um sistema que receba e devolva o pacote. O tipo da mensagem deverá ser alterado de 'echo' para 'echo_ok'.

[solução](https://github.com/crispim1411/maelstrom_challenges_rust/blob/master/dist-sys-rust/src/bin/echo.rs)

Run:
```
./maelstrom/maelstrom test -w echo --bin dist-sys-rust/target/debug/echo --node-count 1 --time-limit 10 
```

## #2 Unique Id
Utilizando do mesma sistema anterior responder a mensagem 'generate' com 'generate_ok' retornando um Id único global.

[solução](https://github.com/crispim1411/maelstrom_challenges_rust/blob/master/dist-sys-rust/src/bin/unique_id.rs)

Run:
```
./maelstrom/maelstrom test -w unique-ids --bin dist-sys-rust/target/debug/unique_id --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
```


## #3 Broadcast
Implementar um sitema de broadcast para uma mensagem circular entre todos os nodes do cluster.

[solução](https://github.com/crispim1411/maelstrom_challenges_rust/blob/master/dist-sys-rust/src/bin/broadcast.rs)

#### 3a - Single-Node
Primeiro cenário apenas considerando um sistema de um Node.

Run:
```
./maelstrom/maelstrom test -w broadcast --bin dist-sys-rust/target/debug/broadcast --node-count 1 --time-limit 20 --rate 10
```

#### 3b - Multi-Node
Considerando que as mensagens enviadas sejam compartilhadas pelos Nodes da estrutura.

Run:
```
./maelstrom/maelstrom test -w broadcast --bin dist-sys-rust/target/debug/broadcast --node-count 5 --time-limit 20 --rate 10
```
