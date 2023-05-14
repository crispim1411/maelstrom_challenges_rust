# Maelstrom ChaUllenges
Solução dos exercícios de Sistemas Distribuidos disponíveis em [fly.io/dist-sys](https://fly.io/dist-sys/)

## #1 Echo
Implementar um sistema que receba e devolva o pacote. O tipo da mensagem deverá ser alterado de 'echo' para 'echo_ok'.

[solução](https://github.com/crispim1411/maelstrom_challenges_rust/blob/master/echo/src/main.rs)

Run:
```
./maelstrom/maelstrom test -w echo --bin dist-sys-rust/target/debug/echo --node-count 1 --time-limit 10 
```

## #2 Unique Id
Utilizando do mesma sistema anterior responder a mensagem 'generate' com 'generate_ok' retornando um Id único global.

[solução](https://github.com/crispim1411/maelstrom_challenges_rust/blob/master/unique_id/src/main.rs)

Run:
```
./maelstrom/maelstrom test -w unique-ids --bin dist-sys-rust/target/debug/unique_id --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
```


## #3 Broadcast
Implementar um sitema de broadcast para a fofoca circular entre todos os nodes do cluster.

[solução](https://github.com/crispim1411/maelstrom_challenges_rust/blob/master/broadcast/src/main.rs)

### 3a - Single Node
Primeiro cenário apenas considerando um sistema de um Node.

Run:
```
./maelstrom/maelstrom test -w broadcast --bin dist-sys-rust/target/debug/broadcast --node-count 1 --time-limit 20 --rate 10
```

