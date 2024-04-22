Formato das mensagens (em 8 bits)
[0, 1] -> sender_id
[2, 3] -> receiver_id
[4, 11] -> timestamp
12 -> message_type
[13, 20] -> message_length

Tipos válidos
Text = 0 - Envia mensagem de texto
File = 1 - Envia arquivo em binário
ListClients = 2 - Solicita uma mensagem do servidor contendo json que lista os clients conectados, seus ids e nomes atribuidos
SetName = 3 - Faz uma requisição ao servidor para alterar o nome do client