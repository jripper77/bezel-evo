# FPS de jogos

[English](../fps.md)

No Windows, o Bezel lê o serviço Intel PresentMon instalado separadamente,
com RTSS como alternativa para FPS. O Bezel não injeta código no jogo.

## Ativar FPS de jogos

1. Selecione **Ativar FPS dos jogos** no instalador, ou **Setup game FPS** no
   menu Iniciar. Na versão portátil, execute `setup-game-fps.cmd`.
2. Aprove uma vez a instalação do coletor Intel assinado. Studio e Light
   continuam como usuário normal. Não há alteração de grupos nem necessidade
   de sair e entrar no Windows.
3. Adicione um sensor FPS ao tema e coloque o jogo em primeiro plano. A escolha
   do processo é uma heurística de aplicativo gráfico, não uma lista de jogos.

O componente inclui serviço e API PresentMon 2.6.0, sem a interface Intel.
Um serviço Intel compatível é reutilizado; serviços incompatíveis são
preservados e o instalador informa o conflito. Se a instalação falhar ou for
recusada, use **Setup game FPS** para tentar novamente. RTSS permanece como
alternativa quando disponível.

A leitura usa um worker e só solicita quadros quando há sensores FPS ativos.
Os dados mantêm o tempo QPC de origem e expiram após três segundos. Parar as
solicitações fecha apenas a sessão do cliente, sem encerrar o serviço compartilhado.
Tentativas sem quadros param após 10 segundos inicialmente, ou após 60 segundos
de pausa depois de medir. Voltar ao jogo elimina a espera ociosa; falhas de
conexão mantêm uma espera limitada entre tentativas.

O componente tem uma entrada própria **Bezel Evo - Game FPS support (Intel
PresentMon)** em Aplicativos do Windows. Remover o Bezel preserva esse serviço,
que pode ser usado por outros programas. Feche os clientes antes de removê-lo.
Arquivos modificados ou desconhecidos, licenças e script de manutenção são mantidos.

### Avançado: console antigo

Um `presentmonPath` explícito no `settings.json`, ou `--presentmon-path` na CLI,
seleciona o console antigo em vez do serviço. Essa opção ainda exige direitos
ETW (administrador ou Performance Log Users). Remova a opção para usar o serviço.
O Bezel não eleva o editor automaticamente nem altera grupos do Windows.

## Se o contador continuar indisponível

Verifique se o suporte FPS está instalado, se o tema usa um sensor FPS e se o
jogo está renderizando em primeiro plano. Jogos pausados ou minimizados podem
parar de produzir quadros. Use **Setup game FPS** se o serviço estiver ausente
ou parado.

Se o PresentMon independente também não receber quadros, salve o trabalho e
tente reiniciar o Windows antes de reinstalar o Bezel ou alterar permissões.
Em um teste local de DRG Survivor, ambos os leitores falharam e a captura
independente informou perda de eventos ETW. O usuário confirmou a recuperação
após reiniciar o Windows. Isso não identifica a causa nem garante a mesma
solução para todas as falhas.

## Sensores

| Chave | Significado |
|---|---|
| `gpu.fps` | FPS médio no último segundo medido, usando intervalos de presents quando disponíveis |
| `gpu.frametime` | Intervalo médio entre quadros no último segundo medido, em ms |
| `gpu.fps_low_1p` | Inverso da média dos 1% de intervalos mais lentos em até 30 segundos |
| `gpu.fps_low_01p` | O mesmo cálculo para 0,1% |
| `gpu.fps_min` / `gpu.fps_max` | Menor / maior média de FPS por segundo em até 30 segundos de histórico |
| `gpu.game` | Nome do executável medido |
| `gpu.fps.source` | PresentMon ou RTSS |
| `gpu.limiter` | Estimativa conservadora CPU / GPU / limite de FPS; indisponível se ambígua |

O coletor entrega quadros em lotes. A janela termina no quadro mais recente
recebido, usando seu tempo de origem, e não o momento da leitura pelo Bezel.
A leitura expira quando esse quadro tem mais de três segundos, mesmo se a
saída acabou de chegar. O valor não é uma contagem instantânea do monitor.

Mínimo e máximo usam janelas completas de um segundo com pelo menos dois
quadros. Ficam indisponíveis até existir uma janela completa; janelas parciais
são excluídas. Não usam o inverso do quadro individual mais rápido: um intervalo
de 0,2 ms pareceria 5.000 FPS. Os lows abaixo continuam sendo estatísticas dos
intervalos entre quadros, que descrevem outro aspecto do ritmo de apresentação.

Os lows precisam de 100 quadros para 1% e 1000 para 0,1% dentro da janela.
Antes disso, o Studio informa que está coletando dados. Uma taxa muito baixa
pode nunca fornecer 1000 quadros em 30 segundos. Perder quadros por excesso
na fila torna as estatísticas de histórico indisponíveis até recompor a janela.

Esses FPS não garantem contagens separadas de quadros nativos e gerados.
Console e serviço foram comparados em Windrose com geração de quadros ligada
e desligada; as métricas do serviço não comprovaram essa separação na
configuração testada. Não divida o resultado por um multiplicador presumido.

## Alternativa RTSS

Inicie RTSS (separadamente ou com MSI Afterburner) e o jogo. Ele fornece FPS
do aplicativo medido mais recentemente. As outras métricas novas ficam
indisponíveis; `gpu.fps.source` mostra RTSS. Se nenhuma fonte puder ler, o
Studio explica o motivo e preserva o detalhe original do diagnóstico.

Zero informado por RTSS continua zero. Dados ausentes ou antigos ficam
indisponíveis; o Bezel não conserva o último valor para sempre. As métricas
PresentMon de um segundo expiram mesmo que o processo continue aberto.
