# Maelstrom ChaUllenges
Solução dos exercícios de Sistemas Distribuidos disponíveis em [fly.io/dist-sys](https://fly.io/dist-sys/)

Baseado no vídeo do Jon Gjengset ([link](https://www.youtube.com/watch?v=gboGyccRVXI))

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
Considerar que as mensagens enviadas sejam compartilhadas pelos Nodes da estrutura.

Run:
```
./maelstrom/maelstrom test -w broadcast --bin dist-sys-rust/target/debug/broadcast --node-count 5 --time-limit 20 --rate 10
```

#### 3c - Fault Tolerant
Considerar que os Nodes possam ficar sem se comunicar por certos períodos de tempo. 

Run:
```
./maelstrom/maelstrom test -w broadcast --bin dist-sys-rust/target/debug/broadcast --node-count 5 --time-limit 20 --rate 10 --nemesis partition
```

#### 3d e 3e - Efficient Broadcast
Considerando tanto multi-node quanto cenário de fault tolerant, agora será requerido que o sistema esteja dentro de algumas métricas. 

Os resultados do código foram:

|Métricas|Esperado|Resultado |
|--------|--------|----------|
| Messages-per-operation |  < 30    | 4    |
| Latência média         | < 400 ms | 4ms  |
| Latência máxima        | < 600 ms | 15ms |

Run:
```
./maelstrom/maelstrom test -w broadcast --bin dist-sys-rust/target/debug/broadcast --node-count 25 --time-limit 20 --rate 100 --latency 100 --nemesis partition
```
