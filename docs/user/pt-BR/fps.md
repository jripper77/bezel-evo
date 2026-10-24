# FPS de jogos

[English](../fps.md)

Os sensores de FPS (`gpu.fps` e métricas relacionadas) mostram os quadros por
segundo do jogo que você está jogando. O Bezel não se injeta nos jogos: no
Windows ele lê o Intel PresentMon e, como alternativa, o que o RivaTuner
Statistics Server, uma ferramenta de FPS que você talvez já use, publica.
Arraste um sensor para um tema como qualquer outro.

> As leituras seguem os formatos publicados dessas ferramentas e foram
> conferidas com um jogo rodando em hardware real (18-10-2026). Se algum valor
> parecer errado, avise.

## Windows: Intel PresentMon (Preferencial)

O Bezel pode ler métricas de quadros diretamente da aplicação de console do
Intel PresentMon. Ele inicia o PresentMon sob demanda, apenas enquanto um jogo
está em execução e um tema solicita os sensores, e o encerra cerca de 60
segundos após o último quadro. Ele nunca é instalado como um serviço.

1. Baixe a aplicação de console do Intel PresentMon (v2.6.0 ou superior).
   Tanto o `PresentMon.exe` quanto nomes de release com versão, como
   `PresentMon-2.6.0-x64.exe`, funcionam.
2. Coloque-o em um destes locais (verificados nesta ordem):
   - O caminho configurado no Bezel: a chave `presentmonPath` no
     `settings.json`, ou `--presentmon-path` na linha de comando.
   - Uma pasta `tools\` ou `sensors\` ao lado do executável do Bezel.
   - `C:\Program Files\Intel\PresentMon\`.
3. **Direitos de administrador necessários**: o PresentMon lê eventos ETW.
   Execute o Bezel como administrador, ou adicione seu usuário ao grupo
   "Performance Log Users". Se os direitos faltarem, os sensores aparecem
   como indisponíveis com uma mensagem explicando a correção.
4. Abra o jogo. O Bezel inicia o PresentMon automaticamente, exclui os
   processos de compositor/overlay (como `dwm.exe`, `explorer.exe`,
   `discord.exe`) e mostra `gpu.fps`, `gpu.frametime`, `gpu.fps_low_1p`,
   `gpu.fps_low_01p`, `gpu.fps_min`, `gpu.fps_max`, `gpu.fps.source`,
   `gpu.game` e `gpu.limiter`.

Se o PresentMon não for encontrado ou falhar ao iniciar, o `gpu.fps` volta
ao RivaTuner Statistics Server (RTSS) exatamente como antes, e o catálogo
indica qual fonte está ativa.

## Windows: RivaTuner Statistics Server (Fallback)

1. Instale o RivaTuner Statistics Server (RTSS). Ele vem com o MSI Afterburner,
   ou sozinho.
2. Abra o RTSS e deixe-o rodando (ele pode ficar na bandeja).
3. Abra o jogo. O Bezel mostra o FPS do último jogo que o RTSS mediu.

Com o RTSS fechado, o sensor fica indisponível e diz isso ("RivaTuner
Statistics Server is not running: start it ...").

## O que aparece

- Um número enquanto o jogo roda. Um 0 informado pela ferramenta aparece como 0.
- `—` quando nenhuma ferramenta está medindo um jogo, ou quando o último valor
  tem mais de 3 segundos (jogo pausado, minimizado ou fechado). O Bezel nunca
  fica mostrando um valor velho.
