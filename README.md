Formato das mensagens (em 8 bits)
[0, 1] -> placeholder
[2, 3] -> receiver_id
[4, 11] -> timestamp
12 -> message_type
[13, 20] -> message_length

Tipos válidos
Text = 0,
File = 1