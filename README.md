# Maelstrom Challenges
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
Implementar um sistema simples de echo que receba e devolva o pacote alterando apenas o tipo da mensagem para 'echo_ok'.

[solução](https://github.com/crispim1411/maelstrom_challenges_rust/blob/master/dist-sys-rust/src/bin/echo.rs)

|System specification |   |
|:-------------------:|:-:|
| Nodes      | 1 |
| Time limit | 10s |

## #2 Unique Id
Implementar um sistema que gere Id únicos. O sistema deverá continuar operando durante falhas na rede, servindo sempre com total disponibilidade.

[solução](https://github.com/crispim1411/maelstrom_challenges_rust/blob/master/dist-sys-rust/src/bin/unique_id.rs)

|System specification |   |
|:-------------------:|:-:|
| Nodes        | 3 | 
| Time limit   | 30s |
| Message rate | 1000 per second |
| Nemesis      | Partition Network |
| Check        | Total avaibility |

## #3 Broadcast
Implementar um sitema de broadcast para uma mensagem circular entre todos os nodes do cluster.

[solução](https://github.com/crispim1411/maelstrom_challenges_rust/blob/master/dist-sys-rust/src/bin/broadcast.rs)

#### 3a - Single-Node
Primeiro cenário apenas considerando um sistema de um Node.

|System specification |   |
|:-------------------:|:-:|
| Nodes        | 1 | 
| Time limit   | 20s |
| Message rate | 10 per second |

#### 3b - Multi-Node
Considerar que as mensagens enviadas sejam compartilhadas pelos Nodes da estrutura.

|System specification |   |
|:-------------------:|:-:|
| Nodes        | 5 | 
| Time limit   | 20s |
| Message rate | 10 per second |

#### 3c - Fault Tolerant
Considerar que os Nodes possam ficar sem se comunicar por certos períodos de tempo devido falhas na rede. 

|System specification |   |
|:-------------------:|:-:|
| Nodes        | 5 | 
| Time limit   | 20s |
| Message rate | 10 per second |
| Nemesis      | Partition Network |

#### 3d e 3e - Efficient Broadcast
O sistema será posto à prova, sendo requerido que esteja dentro de algumas métricas de latência. 

|System specification |   |
|:-------------------:|:-:|
| Nodes        | 25 | 
| Time limit   | 20s |
| Message rate | 100 per second |
| Latency      | 100ms |
| Nemesis      | Partition Network (optional) |

Abaixo os resultados obtidos:

|Métricas|Esperado|Resultado|Com Network Partition|
|--------|:------:|:-------:|:-------------------:|
| Messages-per-operation |  < 30   | 4     | 4     |
| Latência média         | < 400ms | 1.5ms | 1.6ms |
| Latência máxima        | < 600ms | 2.3ms | 8.8ms |
