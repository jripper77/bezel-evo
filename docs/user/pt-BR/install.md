# Instalar o Bezel Evo

[English](../install.md)

O Bezel Evo roda no **Windows 10 e 11, x64**. Baixe-o na
[página de releases](https://github.com/jripper77/bezel-evo/releases):

| Arquivo | O que é |
|---|---|
| **Setup** (`Bezel-Evo-<versão>-windows-x64-setup.exe`) | Instala o Studio, o Light, os temas inclusos e o leitor de sensores de hardware para o seu usuário do Windows |
| **ZIP portátil** (`…-windows-x64-portable.zip`) | Os mesmos programas, para extrair numa pasta onde você possa gravar |
| **Checksums** (`…-SHA256SUMS.txt`) | O SHA-256 de cada arquivo, para conferir um download |
| **Source** | O código-fonte dessa release |

Antes de instalar, feche qualquer outro programa que controle a tela (o app do
fabricante, o turing-smart-screen-python): só um programa pode usá-la por vez.
Veja [Vindo do turing-smart-screen-python](migrating.md).

## Setup

Rode o Setup. Ele instala para o seu usuário do Windows e, na página
**Startup and hardware sensors** (o Setup é em inglês), oferece duas opções:

- **Start Bezel Evo Light when I sign in** (iniciar o Light ao entrar): os seus temas salvos voltam às telas quando
  você entra, sem abrir o Studio ([Iniciar com o Windows](run-at-login.md)).
- **Configure hardware sensors** (configurar os sensores): registra o leitor de sensores incluso, para ler
  temperaturas da CPU, ventoinhas e potência ([Sensores](sensors.md)). Só esse
  passo pede aprovação de administrador; aprove com a mesma conta do Windows. O
  Studio e o Light sempre rodam como o seu usuário normal.

Se o Microsoft Edge WebView2 faltar, o Setup o baixa. Os sensores de hardware
usam o .NET Framework 4.7.2 ou mais novo, que o Windows 10 e 11 já trazem.

Depois abra o **Bezel Studio** pelo menu Iniciar e
[crie o seu primeiro tema](first-theme.md).

## ZIP portátil

Extraia o ZIP numa pasta onde você possa gravar e abra `bezel-studio.exe`. A
versão portátil precisa do WebView2 já instalado.

- Rode `setup-evo-sensors.cmd` uma vez para configurar o leitor de sensores de
  hardware.
- Dê dois cliques em `start-light.cmd` para iniciar o Light, depois de salvar um
  tema e escolher uma tela no Studio.

## Instaladores sem assinatura

O Setup e os programas do Bezel Evo não têm assinatura digital, então o Windows
SmartScreen pode parar o Setup com "O Windows protegeu o computador". Confira se
o arquivo é o que você baixou, clique em **Mais informações** e depois em
**Executar assim mesmo**.

Para conferir se um download chegou inteiro, compare o SHA-256 dele com a linha
desse arquivo em `Bezel-Evo-<versão>-SHA256SUMS.txt`, na mesma release
(maiúsculas ou minúsculas tanto faz):

```powershell
Get-FileHash .\Bezel-Evo-<versão>-windows-x64-setup.exe
```

## Atualizar

Uma vez por dia o Light pergunta ao GitHub qual é a última versão publicada
(envia só a própria versão, nada mais). Quando existe uma mais nova, o menu da
bandeja mostra **Atualização disponível: X.Y.Z** e o Studio mostra em
**Preferências › Atualizações**, com um botão para a página de download. Nada é
baixado nem instalado por você; a chave no mesmo lugar desliga a verificação.

Salve o seu tema e feche o Studio, depois feche o Light pelo ícone da bandeja
(**Quit**). Rode o novo Setup, ou extraia o novo ZIP por cima da pasta antiga. O
Setup se recusa a atualizar com o Studio ou o Light abertos; ele nunca os fecha
por você.

As atualizações mantêm os seus temas, preferências, fontes e caches. Uma
atualização silenciosa (`/S`) mantém as opções escolhidas antes e não liga
opções novas.

## Desinstalar

Use **Configurações → Aplicativos → Aplicativos instalados** no Windows. A
desinstalação remove os programas, e a tarefa de sensores e a entrada do Light
na inicialização que pertencem a esta instalação. Os seus temas, preferências e
caches ficam.

## O que não vem incluso

- O **ffmpeg**, para converter vídeos: [Instalar o ffmpeg](ffmpeg.md).
- **Drivers de hardware**: a maioria das telas não precisa de nenhum; a geração
  Turing USB e os painéis WCH precisam do WinUSB
  ([Deixe o Bezel abrir a tela](permissions.md)).

Detalhes para quem mantém o projeto, instalação silenciosa e os scripts da
versão portátil estão no
[guia de empacotamento para Windows](../../../packaging/windows/README.md) (em
inglês). Para compilar o código-fonte, veja
[Build on Windows](../../build-windows.md).
