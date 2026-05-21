## OClapp

oClapp is a Rust based app that builds functionalities very similar to omlx @omlx-ARCHITECTURE.md
 but it uses Llama.cpp under the hood.

 Features I need  are
 - Desktop app that allow me to show list of recommended models from Hugginface, (use can also diretly copy and paste hugging face model name)
 - User can define a folder where all local models are saved
 - User can download models locally and start a coding agent lke codex, cluade or pi using a simple launch commans  like
 ```
$ oClapp launch claude --model <modelname>
$ oClapp launch pi --model <modelname>

```
 - We should provide an interface that allows user to easily tweak the various llamacpp settings for the model (without overwhelming the user)
 - I believe we will need to spin up a server to expose the endpoint
 - I would prefer th stack stack for the app to be  Rust, and Tauri
