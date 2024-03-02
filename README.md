# sergeant
Sergeant is a terminal based Twitch chat widget and bot in one. It will display the Twitch chat from your channel in terminal while also allowing you to add chat bot commands.

# Installation
```
$ cargo install sergeant
```

# Usage
## sergeant chat
Displays your Twitch channel chat in your terminal. Authentication is required, the easiest way is to use `sergeant login`. You can also use positional arguments to provide your twitch screen name, an oauth token, and a client id. You can also provide environment variables. `sergeant chat --help` for more details. 

## sergeant login
Starts an OAuth login flow to get a token. Navigate to the URL it prints to the terminal, it will wait for you to authenticate. Once complete you're ready to use `sergeant`.

## sergeant commands
Use this to add, remove, and list chat commands.
```
# add !today command:
sergeant commands add today "Today I am going to rust all day long!"

# remove command:
sergeant commands remove today

# list commands:
sergeant commands list

# add a recurring announcement every 5 mins:
sergeant commands add spam "Spam your Twitch channel all you want" 5
```
