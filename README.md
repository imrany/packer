Packer is a simple web server used to serve static contents. 

## Configuring Packer
Open packer folder, go to `config.json`
```json
{
    "server_name":"My movie server", // specify the name of your server
    "location":"C:/Users/username/Downloads", // specifies the path location of the static content you want to serve, eg "/template/web" or "C:/Users/username/web"
    "listen":80 // specifies the port you want to serve on, by defaut is port 80
}
```
