import { TCPHelper} from "@companion-module/base"
import { Config } from "./config"

export const sendCommand = (cmd: string, config: Config) => {
    if (config.vmix_ip) {
        let socket = new TCPHelper(config.vmix_ip, 8099)

        socket.on('error', (err) => {
            console.log(err)
            //updateStatus(InstanceStatus.ConnectionFailure, err.message)
            //log('error', 'Network error: ' + err.message)
        })

        socket.on('data', (data) => {
            if (data.toString().includes('VERSION')) {
                socket.send('PING\r\n')
                socket.send(cmd)
                socket.send('QUIT\r\n')
            }
            if (data.toString().includes("QUIT OK Bye")) {
                socket.destroy()
            }
        })
        console.log('Trying to send command')
        
    } else {
        console.log("failed at sending command")
    }

    
}