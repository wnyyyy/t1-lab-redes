Formato das mensagens (em 8 bits)
[0, 1] -> sender_id
[2, 3] -> receiver_id
[4, 11] -> timestamp
12 -> message_type
[13, 20] -> message_length
[21, 22] -> ID mensagem UDP (usado para construir mensagens por UDP)

Tipos válidos
Text = 0 - Envia mensagem de texto. Conteúdo da mensagem possui o texto a ser enviado.
File = 1 - Envia arquivo em binário. Conteúdo da mensagem possui os dados a serem enviados.
ListClients = 2 - Solicita uma mensagem do servidor contendo json que lista os clients conectados, seus ids e nomes atribuidos.
SetName = 3 - Faz uma requisição ao servidor para alterar o nome do client. O servidor retorna uma mensagem contendo 0 (Falha) ou 1 (Sucesso) para se trocou o nome ou não.