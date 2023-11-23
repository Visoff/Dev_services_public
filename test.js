const net = require('net');

const server = net.createServer((socket) => {
  socket.once('data', (data) => {
    const requestString = data.toString();
    const requestLines = requestString.split('\r\n');

    // Parse the request method, URI, and HTTP version
    const [method, uri] = requestLines[0].split(' ');

    // Parse the headers
    const headers = {};
    for (let i = 1; i < requestLines.length; i++) {
      if (requestLines[i] === '') {
        // Headers end, and the body starts after an empty line
        const body = requestLines.slice(i + 1).join('\r\n');
        const jsonResponse = {
          method,
          uri,
          headers,
          body,
        };

        if (headers["Content-Type"] == "application/json") {
          jsonResponse.body = JSON.parse(jsonResponse.body)
        }

        console.log("resuest");
        
        // Send the response as JSON
        socket.write('HTTP/1.1 200 OK\r\n');
        socket.write('Content-Type: application/json\r\n');
        socket.write('Hiii: header_value\r\n');
        socket.write('\r\n');
        socket.write(JSON.stringify(jsonResponse));
        socket.end();

        break;
      } else {
        const [key, value] = requestLines[i].split(': ');
        headers[key] = value;
      }
    }
  });
});

const port = 8081;

server.listen(port, () => {
  console.log(`Server listening on port ${port}`);
});
