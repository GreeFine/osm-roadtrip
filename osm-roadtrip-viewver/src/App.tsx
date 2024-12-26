import { useEffect, useState } from 'react';
import './App.css';

function App() {
  const [svg, setSvg] = useState("");

  useEffect(() => {
    (async () => {
      const svg_request = await fetch("http://localhost:8080/svg");
      if (svg_request.ok) {
        const api_svg = await svg_request.text();
        setSvg(api_svg);
      }
    })()
  }, [])

  const size = svg.length;

  return (
    <div className="App">
      <p>Size of svg: {size}</p>
      <div dangerouslySetInnerHTML={{ __html: svg }} />
    </div>
  );
}

export default App;
