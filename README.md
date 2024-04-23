Formato das mensagens (em 8 bits)

[0, 1] -> key (Identificador atribuido pelo client para vincular a mensagem enviada a uma resposta do servidor)

[2, 3] -> receiver_id

4 -> message_type

[5, 12] -> message_length

[13, 14] -> ID mensagem UDP (usado para construir mensagens por UDP)

[15, 16] -> Sequência do pacote UDP (usado para construir mensagens por UDP)

Tipos de mensagem

Connection = 0 - Requisição feita pelo client para se conectar ao servidor. Conteúdo da mensagem pode possuir o nome do
client. // Servidor responde com uma mensagem tipo 7, Error, contendo motivo da falha; ou 8, Success, contendo o id do
client.

Text = 1 - Envia mensagem de texto. Conteúdo da mensagem possui o texto a ser enviado. // Servidor responde com uma
mensagem tipo 7, Error, contendo motivo da falha (eg, destinatário offline); ou 8, Success. Quem recebe a mensagem,
recebe do servidor uma mensagem deste tipo com o conteúdo da mensagem, mas com receiver_id sendo sender_id (id do
remetente).

File = 2 - Envia arquivo em binário. Conteúdo da mensagem possui os dados a serem enviados. // Servidor responde com uma
mensagem tipo 7, Error, contendo motivo da falha (eg, destinatário offline); ou 8, Success.

ListClients = 3 - (Somente Header) Solicita uma mensagem do servidor contendo json que lista os clients conectados, seus
ids e nomes
atribuidos. // Servidor responde com uma mensagem tipo 8, Success, contendo um json com as informações.

SetName = 4 - Faz uma requisição ao servidor para alterar o nome do client. // Servidor responde com uma mensagem tipo
7, Error, contendo motivo da falha (eg, nome já existente); ou 8, Success.

Broadcast = 5 - Envia uma mensagem para todos os clients conectados. Conteúdo da mensagem possui o texto a ser
enviado. // Servidor responde com uma mensagem tipo 8, Success.

Disconnect = 6 - (Somente Header) Solicita ao servidor para desconectar o client. // Servidor responde com uma mensagem
tipo 8, Success.

Error = 7 - Mensagem enviada pelo servidor para retornar um erro. Conteúdo da mensagem possui o motivo do erro.

Success = 8 - Mensagem enviada pelo servidor para retornar sucesso. Conteúdo da mensagem possui o id do client, ou um
json

Ping = 9 - (Somente Header) Mensagem enviada periodicamente pelo servidor para verificar se o client está ativo. Client
deve responder com outro ping.

