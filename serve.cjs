const http = require('http');
const fs = require('fs');
const path = require('path');

const dir = path.resolve(__dirname, 'target/web/dist');
const port = parseInt(process.env.PORT || process.argv[2] || '8770', 10);

const mimeTypes = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript',
  '.wasm': 'application/wasm',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.json': 'application/json',
};

const server = http.createServer((req, res) => {
  res.setHeader('Cross-Origin-Opener-Policy', 'same-origin');
  res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp');

  let reqPath = req.url.split('?')[0].split('#')[0];
  if (reqPath === '/' || reqPath === '') reqPath = '/index.html';
  const filePath = path.join(dir, reqPath.replace(/^\/+/, ''));

  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404, { 'Content-Type': 'text/plain' });
      res.end('Not Found');
      return;
    }
    const ext = path.extname(filePath);
    res.writeHead(200, {
      'Content-Type': mimeTypes[ext] || 'application/octet-stream',
      'Content-Length': data.length,
    });
    res.end(data);
  });
});

server.listen(port, '127.0.0.1', () => {
  console.log(`Serving ${dir} on http://127.0.0.1:${port}/`);
});
