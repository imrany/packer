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

## Usage
To run with configurations specified in  `config.json`, use
```bash
packer
```

To serve a contents from a folder, use
`packer serve <path of the folder you want to serve>
` example, to serve contents on the current folder, run
```bash
packer serve ./
```
to serve downloads folder, run
```bash
packer serve "C:/Downloads"
```

To serve on a specified port, use
`packer --port=<number> serve "C:/Downloads"` replace `<number>` with a any number e.g: 3000, 80, 443 e.t.c
example
```bash
packer --port=3000 serve ./
```

## Downloads
|   Operating systems |     Prebuilt Binaries            |
|-----------------------|------------------------------|
|  Windows 64Bit        | [packer-v0.2.2-win64.zip](https://github.com/imrany/packer/releases/download/v0.2.2/packer-v0.2.2-win64.zip)  |

## Source code
Built from source [packer source code](https://github.com/imrany/packer/archive/refs/tags/v0.2.2.zip)
