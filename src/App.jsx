import React from 'react';
import MarketDepth from './components/MarketDepth';
import './App.css';

function App() {
    return (
        <div className="App">
            <header className="App-header">
                <h1>High-Performance Market Depth Demo</h1>
                <p>Powered by React + WebAssembly</p>
            </header>
            <main>
                <MarketDepth />
            </main>
        </div>
    );
}

export default App;