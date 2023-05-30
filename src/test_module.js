const net = require('net');

function sendData(host, port, data) {

    //console.log(data)
    const client = net.createConnection({ host: host, port: port }, () => {
        client.write(data);
    });

    client.on('data', (data) => {
        //console.log(data.toString());
        client.end();
    });

    client.on('end', () => {
        console.log('disconnected from server');
    });
}

module.exports = { sendData };
