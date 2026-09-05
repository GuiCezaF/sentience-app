# Sentience App

Agente de bandeja que classifica expressão facial localmente e sincroniza só o resultado. O frame de câmera não é um dado do produto: existe só para produzir uma Classificação.

## Language

**Classificação**:
Registro de negócio de um recorte válido e acima do Piso de confiança: `classification_id` (UUID da Captura, estável no reenvio), Sujeito, instante UTC e Emoção vencedora. Não inclui Frame, Recorte de face nem o vetor de probabilidades no Gateway.
_Avoid_: detecção de emoção, sample, dado, informação, inferência (como substantivo de negócio)

**Emoção**:
Uma das quatro classes do artefato ONNX, nesta ordem, e nesta grafia no Lote: `angry`, `happy`, `neutral`, `sad`. O Extra de bandeja traduz para pt-BR.
_Avoid_: sentimento, humor, mood, estado, rótulo em português no fio

**Piso de confiança**:
Probabilidade mínima do argmax (0,45) para nascer Classificação. Abaixo disso o tick é Gap, igual a sem recorte.
_Avoid_: accuracy, qualidade, score (como sinônimo solto)

**Sujeito**:
Identificador de usuário colocado no envio. Nesta fase é injetado na simulação (constante / ambiente). No contrato: `subject_id`.
_Avoid_: account, cliente, device (como se fosse a pessoa)

**Sessão de câmera**:
Stream contínuo aberto enquanto o processo vive na bandeja. O LED permanece aceso de propósito, para não piscar.
_Avoid_: captura contínua, gravação, recording

**Captura**:
Extração de um único frame da Sessão de câmera a cada 5 segundos, para classificar. Não é vídeo salvo.
_Avoid_: gravação, snapshot persistido, frame (como registro)

**Frame**:
Imagem efêmera da câmera. Depois da Classificação (ou da decisão de não classificar), é descartado. Nunca entra no armazenamento local nem na Sincronização.
_Avoid_: foto, evidência, mídia

**Recorte de face**:
Entrada do modelo: 48×48, um canal, uint8, rosto já recortado. Sem Recorte de face válido não existe Classificação. Se houver várias faces, usa-se a maior ou a mais central; o resultado é atribuído ao Sujeito.
_Avoid_: frame (como input do ONNX), imagem, thumbnail

**Gap**:
Intervalo de tempo sem Classificação (sem recorte válido ou abaixo do Piso de confiança). Não é um evento e não entra no Lote.
_Avoid_: ausência (como registro), no-face event, Classificação nula

**Saúde**:
Condição operacional enviada em todo POST de Sincronização: `ok`, `camera` ou `model`. Independente de haver Classificações no Lote.
_Avoid_: status da UI como se fosse o campo do Gateway, Falha (como valor do fio)

**Extra de bandeja**:
Única superfície do produto: cartão claro estilo extra de menu / Control Center, sem dark mode neste ciclo. Some fora de foco. Preview de câmera só enquanto o cartão está visível; fechado, a Sessão de câmera segue para Captura sem renderizar vídeo. Estados: `Ativo` / `Sem recorte` / `Falha`, mais uma linha própria da Sincronização (`pendente` / `ok` / `erro`). A Emoção em pt-BR só aparece se a Captura mais recente virou Classificação; senão o rótulo some.
_Avoid_: dashboard, Settings, app principal

**Lote**:
Conjunto de Classificações ainda não reconhecidas pelo Gateway. `items` vazio não é Lote: é Pulso.
_Avoid_: payload genérico, telemetria, agregado (este app não agrega)

**Pulso**:
Mesmo envio da Sincronização com `items` vazio: o agente está vivo. Carrega Saúde. Não é Gap e não é Classificação.
_Avoid_: heartbeat (como registro de negócio), ausência, ping genérico, item sentinela

**Gateway**:
Destino da Sincronização. Cliente HTTP real, URL configurável; se vazia, stub in-process (2xx, com falha forçável). Um único POST recebe `subject_id`, `sent_at`, `health` e `items`. Sem header de autenticação neste ciclo. Não recebe Frames. O Relatório diário é responsabilidade dele, não deste app.
_Avoid_: backend genérico, servidor, API (como sinônimo solto)

**Sincronização**:
A cada 30 minutos: um POST com o Lote, ou Pulso se não há Classificações pendentes. Sucesso = HTTP 2xx → apaga do local exatamente as Classificações enviadas nesse POST. Falha ou incerteza → não apaga; reenvia os mesmos `classification_id` (at-least-once). O Relatório só importa no dia seguinte.
_Avoid_: backup, upload de mídia, telemetria genérica

**Relatório**:
Visão do dia seguinte gerada fora deste app, a partir do que o Gateway guardou. Fora de escopo aqui.
_Avoid_: dashboard local, histórico no Extra de bandeja
