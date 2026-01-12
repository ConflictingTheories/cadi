import React, { useState, useEffect } from 'react';

export function TimerTodo() {
    const [todos, setTodos] = useState<string[]>([]);
    const [time, setTime] = useState<number>(0);
    const [running, setRunning] = useState<boolean>(false);

    useEffect(() => {
        let id: number | undefined;
        if (running) {
            id = window.setInterval(() => setTime(t => t + 1), 1000);
        }
        return () => { if (id) clearInterval(id); };
    }, [running]);

    function addTodo() {
        setTodos(t => [...t, `Task ${t.length + 1}`]);
    }

    return (
        <div className="todo-app">
            <h1>Todo Timer</h1>
            <div className="timer">Time: {time}s</div>
            <button onClick={() => setRunning(r => !r)}>{running ? 'Pause' : 'Start'}</button>
            <button onClick={addTodo}>Add Todo</button>
            <ul>
                {todos.map((t, i) => <li key={i}>{t}</li>)}
            </ul>
        </div>
    );
}

export default TimerTodo;