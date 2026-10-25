# FPS de jogos

[English](../fps.md)

No Windows, o Bezel pode ler o console Intel PresentMon, com RivaTuner
Statistics Server (RTSS) como alternativa para `gpu.fps`. O Bezel não
injeta código no jogo. Arraste os sensores para o tema.

## Configurar PresentMon

1. Baixe o [console PresentMon 2.6.0 testado](https://github.com/GameTechDev/PresentMon/releases/tag/v2.6.0),
   não o aplicativo gráfico de captura.
2. Configure `presentmonPath` no `settings.json` do Studio, ou
   `--presentmon-path` na CLI. Outra opção é colocar o executável em
   `tools\` ou `sensors\` ao lado do Bezel. A busca também consulta a pasta
   Intel `PresentMon\PresentMonConsoleApplication\`, depois o layout antigo
   `PresentMon\`. Nunca usa `PATH`. Aceita `PresentMon.exe` e nomes como
   `PresentMon-2.6.0-x64.exe`; versões numeradas usam ordem numérica.
3. A captura ETW requer direitos de administrador ou participação no grupo
   **Performance Log Users**. Alterar o grupo pode exigir sair e entrar
   novamente no Windows. O Bezel não concede esses direitos nem se eleva.
4. Solicite um sensor FPS e coloque o jogo em primeiro plano. O Bezel escolhe
   esse PID, filtrando programas comuns do sistema, navegadores e launchers.
   É uma heurística de aplicativo gráfico, não uma lista completa de jogos.

A captura começa sob demanda, sem janela de console. Mede uma swap chain do
PID escolhido, selecionada pela contagem recente de quadros. Outra aplicação
candidata em primeiro plano pode substituir o alvo. Os quadros mantêm o tempo
QPC original: uma fila atrasada não transforma dados antigos em dados recentes.

Parar todas as solicitações FPS encerra o filho. Sem quadros, uma tentativa
para após 10 segundos; depois de medir, tolera até 60 segundos de pausa.
Falhas usam RTSS quando disponível e repetem PresentMon com espera limitada.
Ferramenta ausente repete no máximo uma vez por minuto. Tentativas ociosas
também esperam até um minuto, mas voltar ao jogo em primeiro plano elimina
essa espera ociosa. Uma captura mantida recebe um novo prazo inicial para
receber quadros; leituras antigas continuam indisponíveis até chegarem quadros
novos. Mudar o primeiro plano não elimina a espera por falhas ou permissões.
Uma instância do Bezel possui a sessão por vez; as outras usam a alternativa
em vez de tomar a captura. O filho entra em um job Windows que o encerra ao
fechar antes de concluir a inicialização da captura.

## Sensores

| Chave | Significado |
|---|---|
| `gpu.fps` | FPS médio no último segundo medido, usando intervalos de presents quando disponíveis |
| `gpu.frametime` | Intervalo médio entre quadros no último segundo medido, em ms |
| `gpu.fps_low_1p` | Inverso da média dos 1% de intervalos mais lentos em até 30 segundos |
| `gpu.fps_low_01p` | O mesmo cálculo para 0,1% |
| `gpu.fps_min` / `gpu.fps_max` | Quadro individual mais lento / rápido, expresso em FPS |
| `gpu.game` | Nome do executável medido |
| `gpu.fps.source` | PresentMon ou RTSS |
| `gpu.limiter` | Estimativa conservadora CPU / GPU / limite de FPS; indisponível se ambígua |

O console entrega quadros em lotes. A janela termina no quadro mais recente
recebido, usando seu tempo de origem, e não o momento da leitura pelo Bezel.
A leitura expira quando esse quadro tem mais de três segundos, mesmo se a
saída acabou de chegar. O valor não é uma contagem instantânea do monitor.

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
