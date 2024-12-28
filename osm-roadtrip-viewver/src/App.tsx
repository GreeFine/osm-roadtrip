import React, { useEffect, useRef, useState } from 'react';
import './App.css';


function App() {
  const [svg, setSvg] = useState("");
  const searchParams = window.location.search;

  useEffect(() => {
    (async () => {
      console.log(`http://localhost:8080/svg${searchParams}`);
      const svg_request = await fetch(`http://localhost:8080/svg${searchParams}`);
      if (svg_request.ok) {
        const api_svg = await svg_request.text();
        setSvg(api_svg);
      }
    })()
  }, [])

  const size = svg.length;

  return (
    <div className="App">
      <p>query: {searchParams}</p>
      <p>Size of svg: {size}</p>

    </div >
  );
}

export default App;
