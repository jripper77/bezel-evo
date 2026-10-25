# Solução de problemas

[English](../troubleshooting.md)

Comece pelo seletor de telas do Studio, ou pelo `bezel devices` num terminal:
os dois listam as telas conectadas e o estado delas sem mandar nada para elas.
Acrescente `-v` a qualquer comando para ver o que o Bezel envia e recebe. Quando
precisar de mais detalhes, os [logs](#logs) registram o que o Studio e o Light
fizeram. As mensagens da linha de comando são em inglês; abaixo, cada uma
aparece como o Bezel a escreve.

## "No smart screen found"

- Confira o cabo USB (alguns cabos só carregam) e tente outra porta.
- Telas Turing USB (`1CBE`) e WCH (`43A8`) precisam do driver WinUSB:
  [Deixe o Bezel abrir a tela](permissions.md).
- Se o Windows deu uma porta COM nova à tela, escolha-a de novo no seletor de
  telas do Studio.

## A tela está em uso ("access denied")

Só um programa pode usar uma tela por vez. O Windows recusa uma tela serial que
outro programa está usando, e o Bezel avisa, por exemplo:

```text
access denied to COM5: …
```

Feche esse programa: o turing-smart-screen-python, o app do fabricante, ou um
segundo `bezel run` num terminal. O Studio e o Light dividem as telas entre si
sozinhos. Veja
[Vindo do turing-smart-screen-python](migrating.md#1-pare-o-outro-programa).

## A tela está apagada, ou aparece como "asleep"

As telas rev C dormem quando outro programa as desliga (o
turing-smart-screen-python e o app do fabricante fazem isso ao fechar) e depois
de um `bezel off`. Qualquer coisa que desenhe as acorda: o **Ao vivo** no
Studio, `bezel run`, `bezel show`, `bezel test-pattern`. Acordar leva alguns
segundos.

## A tela travou / parou de responder

Sinais: *"the screen stopped responding: it stopped reading what was sent"* (a
tela parou de responder), "timeout talking to …", um envio ou o tema ao vivo
que empaca. Uma tela Turing rev C (a 8,8" e os outros modelos seriais com chip
de despertar) pode travar, por exemplo com um envio acima do limite de 25 MiB.
**Não precisa desconectar o cabo**: o Bezel a reinicia pelo chip de despertar,
e ela volta em cerca de 10 segundos.

- **Durante o ao vivo**: o Light, o `bezel run` e o **Ao vivo** do Studio
  conectam a tela de novo sozinhos, 2, 5 e 10 segundos depois de
  ela parar; conectar reinicia antes uma tela travada, e o tema continua. A
  barra de status do Studio diz *Reconectando à tela (tentativa 1 de 3)…*,
  depois *A tela voltou e mostra o tema ao vivo de novo.*; o `bezel run`
  escreve o mesmo no terminal. Depois da terceira tentativa o ao vivo para com
  o erro. Desligar o **Ao vivo** (ou Ctrl+C) nesse meio-tempo para na hora.
- **Sozinho**: o próximo comando, ou ligar o **Ao vivo** de novo, encontra a
  tela travada e a reinicia uma vez antes de conectar. Nada é reiniciado
  enquanto a tela responde.
- **Linha de comando**: `bezel restart` (ou `bezel restart -s COM5`
  para uma tela). Ele diz que vai reiniciar a tela, espera por ela e mostra onde
  ela voltou. O terminal também mostra essa dica depois de um erro que indica
  que a tela travou.
- **Studio**: **Tela → Ajustes → Avançado → Reiniciar a tela…**, depois de uma
  confirmação curta. Um erro de armazenamento que indica que a tela travou tem o
  mesmo botão, e o cartão da tela também, quando o ao vivo parou por isso.

Reiniciar para o que a tela mostra ou toca; os arquivos guardados ficam. Se a
tela continuar sem responder depois de reiniciar, ou não tiver chip de
despertar (o Bezel então diz que não consegue reiniciá-la), desconecte o cabo
USB, espere alguns segundos e conecte de novo.

## Depois de cancelar um envio

Apague o arquivo incompleto antes de enviar de novo: o Studio oferece
**Apagar o arquivo incompleto**, e o terminal imprime o comando
`bezel storage rm … --yes`. Se o envio seguinte terminar com *"the stored size
differs; delete it and send it again"*, apague esse arquivo e envie mais uma
vez. Veja [Cancelar um envio](storage-and-video.md#cancelar-um-envio).

## "refused: the file is … MiB and this screen takes files up to 25 MiB each"

As telas Turing rev C aceitam no máximo 25 MiB por arquivo: o firmware guarda o
envio inteiro na memória e trava com um arquivo maior. O Bezel recusa o arquivo
antes de enviar qualquer coisa (no Studio: *"O arquivo tem … MiB; esta tela
aceita arquivos de até 25 MiB."*). Envie um trecho mais curto, ou deixe o Bezel
converter o vídeo com menos quadros por segundo (`--fps 24`): a conversão limita
a taxa de bits para o resultado caber. Veja
[Qual o tamanho máximo de um arquivo](storage-and-video.md#qual-o-tamanho-máximo-de-um-arquivo).

## Um vídeo não é enviado

"needs ffmpeg" ou "ffmpeg não encontrado": instale o ffmpeg com `libx264`
([Instalar o ffmpeg](ffmpeg.md)). Imagens, e vídeos que já estão no formato da
tela, vão sem ele.

## "Nenhum cartão SD na tela"

O cartão não está lá ou não está em FAT32 com tabela de partição MBR:
[Preparar um cartão SD](sd-card.md).

## Um sensor mostra `—`

O `bezel sensors` diz por quê. Veja [Sensores](sensors.md#quando-falta-um-valor).

A barra de status do Studio tem um indicador de sensores: verde quando tudo está
sendo lido, amarelo quando pelo menos um hardware está sem leituras, vermelho
quando nada é lido. Clique nele para ver qual hardware foi afetado. Sensores
opcionais ausentes não deixam amarelo um dispositivo saudável. **Reiniciar
Libre** reinicia o leitor de sensores de hardware incluso; o indicador se
atualiza quando as leituras voltam. Isso pode recuperar uma fonte Corsair cujas
leituras por USB pararam, mesmo quando o Total Output calculado ainda mostra
`0 W`. O reinício precisa da tarefa `Bezel-Sensors` que a opção **Configure
hardware sensors** do Setup (ou o `setup-evo-sensors.cmd` do ZIP portátil) cria.
Veja [O estado dos sensores](sensors.md#o-estado-dos-sensores).

## O FPS de jogos mostra `—`

Nenhuma ferramenta está medindo um jogo, ou o jogo está pausado:
[FPS de jogos](fps.md).

## "O Windows protegeu o computador"

Os instaladores do Bezel não têm assinatura. Clique em **Mais informações** e
depois em **Executar assim mesmo**:
[Instalar o Bezel Evo](install.md#instaladores-sem-assinatura).

## Um painel Turing USB aparece em "desktop mode"

Ele está no modo desktop (segundo monitor) do fabricante:
[Telas suportadas](devices.md#modo-desktop).

## O Studio não abre

O Studio não abre console. Para ver o que ele imprime, abra um Prompt de Comando
na pasta onde o Bezel Evo está instalado e rode
`bezel-studio.exe 2> bezel-studio.txt`; depois abra o `bezel-studio.txt`. Os
[logs](#logs) podem dizer mais.

As mensagens do Studio (`bezel-studio: …`) são frases fixas de uma lista
fechada: nenhuma traz o texto do erro por trás dela, nem um arquivo ou pasta
seus. É de propósito, para que nada que o Studio imprime carregue um segredo,
como a sua chave do KLIPY, ou um caminho pessoal. Quando o Studio para num erro
(um panic), a única linha que ele imprime acrescenta onde no código-fonte isso
aconteceu: um arquivo do código do Bezel ou de uma biblioteca, a partir da pasta
do crate (`tauri-2.…/src/…`), e uma linha e uma coluna, nunca a mensagem do
erro. Quando o Studio não abre, ou abre sem uma parte, ele diz qual parte
falhou:

| Ele diz | O que falhou |
| --- | --- |
| `bezel-studio: the app's folders were not found`, depois `bezel-studio: the app panicked at tauri-…/src/app.rs:…` | Onde o Studio guarda ajustes, dados e cache (as pastas `AppData` do seu usuário). |
| `bezel-studio: the tray icon was not added`, depois `bezel-studio: the app panicked at tauri-…/src/app.rs:…` | O ícone da bandeja. |
| `bezel-studio: the app panicked at tauri-…/src/app.rs:…`, sem linha antes | A janela ou a página dentro dela: o runtime do WebView2. Rode o Setup de novo, ou instale o WebView2 pela Microsoft. |
| `bezel-studio: the app panicked at …`, outro lugar | Um erro dentro do Studio ou de uma biblioteca que ele usa. Relate-o, com a linha inteira. |
| `bezel-studio: the app did not start: a plugin did not start` | Um dos plugins dele: janela única, diálogos de arquivo, abrir links, abrir no login. |
| `bezel-studio: the app did not start` | Outra causa. |
| `bezel-studio: refresh loop not started: no thread or memory to spare` | O Studio abre, mas os sensores e a tela ao vivo não se atualizam: o Windows está sem threads ou sem memória. Feche alguns programas e abra o Studio de novo. |
| `bezel-studio: refresh loop not started` | O mesmo, por outra causa. |

Num lugar, o `…` substitui a versão da biblioteca, a linha e a coluna, que
mudam de uma versão do Bezel para outra. Ao relatar o problema, copie essas
linhas como estão.

## Logs

O Studio e o Light gravam os logs numa pasta só:

```text
%LOCALAPPDATA%\io.github.slipalison.bezel\logs
```

Para abri-la, use **Preferências → Logs → Abrir pasta de logs** no Studio (o
ícone de controles deslizantes na barra de cima). O log do Light é o
`light.log`. Os logs são gravados mesmo com o **Debug** desligado; **Preferências
→ Debug → Ativar logs de diagnóstico** acrescenta detalhes sobre desenho, envios
e sensores enquanto você reproduz um problema. Os logs giram em 2 MiB e nunca
registram pedidos de rede. O leitor de sensores de hardware tem os próprios logs
([guia do leitor de sensores](../../../apps/bezel-sensors-helper/README.md), em
inglês).

## Relatar um problema

Inclua a versão (mostrada no cabeçalho do Studio e pelo `bezel --version`), a
saída de `bezel -v devices` e os [logs](#logs) do momento em que falhou. Tire
números de série e caminhos pessoais antes de publicar.
## Leituras de diagnóstico na tela

Nas preferências do Studio, ative Debug e **Mostrar leituras na tela**, depois
escolha um canto. Ative a transmissão ao vivo para a tela conectada: as leituras
são desenhadas nos quadros enviados ao dispositivo, também quando o Light faz
o envio. Temas estáticos também atualizam a sobreposição, aumentando um pouco
o tráfego USB. Desativar a opção remove as leituras no próximo quadro enviado.

Os números descrevem o FPS de envio do Bezel, os tempos de renderização e
transferência e o tempo de amostragem dos sensores. Não medem a frequência do
painel nem o FPS do jogo. Para o jogo, adicione um [sensor de FPS](fps.md).
O diagnóstico é temporário: não é salvo no tema nem desenhado no canvas do editor.
