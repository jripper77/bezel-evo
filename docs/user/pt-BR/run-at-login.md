# Iniciar com o Windows

[English](../run-at-login.md)

O **Bezel Light** mantém os seus temas salvos nas telas sem o Studio. Ele fica
na bandeja; o menu dele tem **Open Studio** e **Quit**, e um clique duplo no
ícone também abre o Studio.

## O Light ao entrar no Windows

Marque **Start Bezel Evo Light when I sign in** no Setup. Quando você entra, o Light
começa com o tema e as telas que você salvou por último no Studio.

Com o ZIP portátil, dê dois cliques em `start-light.cmd` para iniciar o Light; o
[guia de empacotamento para Windows](../../../packaging/windows/README.md) (em
inglês) tem o comando que o põe na inicialização.

**Tela → Ajustes → Iniciar com o computador**, no Studio, é a mesma opção:
ligá-la ou desligá-la ali faz o mesmo que a caixa do Setup. O próprio Studio
não inicia ao entrar; uma entrada antiga do Studio que fazia isso passa para o
Light na próxima vez que o Studio abrir.

## O Studio e o Light juntos

O Light é o dono das telas: ele roda os temas, lê os sensores, guarda o
histórico dos gráficos e os relógios, toca as transições e decodifica os vídeos
de fundo. O Studio é o editor que conversa com ele.

- **Abrir o Studio** não interrompe nada: a tela mantém a imagem, e a área de
  edição do Studio mostra exatamente o que o Light desenha. As suas mudanças vão
  para o Light enquanto você edita, então uma tela ao vivo as acompanha.
- **Fechar o Studio** mantém tudo andando: nada é redesenhado, os gráficos
  guardam o histórico, um vídeo continua tocando. Se você fechar sem salvar, o
  Studio pergunta antes, e a tela volta ao tema salvo, ainda sem recomeçar.
- Os dois conversam por um canal local que só o seu usuário do Windows pode
  abrir.
- Se o Light não conseguir iniciar ou parar de responder, o Studio controla as
  telas sozinho, como nas versões anteriores.

Fechar o Studio sempre passa as telas para o Light; não há opção para manter
o Studio na bandeja. **Quit** no menu da bandeja fecha o Light; a tela mantém a
última imagem.

## Várias telas

O seletor de telas no alto do Studio abre um documento separado para cada tela.
Trocar de tela mantém as mudanças e o histórico de Desfazer de cada uma, e as
outras telas continuam rodando. Salve um tema e ligue o **Ao vivo** em cada tela
que você quer usar.

O Light roda todas num só processo, com um só leitor de sensores e um trabalho
por tela, então uma tela lenta não segura uma rápida.

## Ao desligar

Quando o Windows desliga ou reinicia, o Light para de desenhar e aplica a
[escolha de desligamento](power-off.md) de cada tela rev C (deixar como está,
apagar, um vídeo guardado ou o álbum de fotos). Telas sem essas escolhas, como a
3,5" rev A, são apagadas. O Light espera no máximo 3,5 segundos; o temporizador
da própria tela é o plano B se uma delas não responder.

Depois de uma pausa longa (suspensão, hibernação), o Light reinicia o leitor de
sensores e reconecta as telas sozinho.

## Sem o Studio nem o Light

O `bezel run` mostra um tema pelo terminal até Ctrl+C:

```powershell
bezel run turing-8.8-horizontal
```

Ele não divide a tela: feche o Light antes, ou ele recusa a porta. Uma tarefa
agendada pode iniciá-lo ao entrar no Windows, mas o Light é o melhor caminho.
