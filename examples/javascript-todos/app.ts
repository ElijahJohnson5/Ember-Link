import { createClient } from '@ember-link/core';
import { createYJSStorageProvider, YjsStorageProvider } from '@ember-link/yjs-storage';

declare global {
  interface EmberLink {
    Presence: {
      isTyping: boolean;
    };
  }
}

async function run() {
  const client = createClient({
    baseUrl: 'http://localhost:9000'
  });

  const { channel } = client.joinChannel<YjsStorageProvider>('todos', {
    initialPresence: { isTyping: false },
    storageProvider: createYJSStorageProvider()
  });

  const whoIsHere = document.getElementById('who_is_here') as HTMLDivElement;
  const todoInput = document.getElementById('todo_input') as HTMLInputElement;
  const someoneIsTyping = document.getElementById('someone_is_typing') as HTMLDivElement;
  const todosContainer = document.getElementById('todos_container') as HTMLDivElement;

  channel.events.subscribe('others', (others) => {
    whoIsHere.innerHTML = `There are ${others.length} other users online`;

    console.log(others);

    someoneIsTyping.innerHTML = others.some((other) => other.isTyping)
      ? 'Someone is typing...'
      : '';
  });

  const storage = channel.getStorage();

  const todos = storage.getArray<{ text: string }>('todos');

  todoInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      channel.updatePresence({ isTyping: false });
      todos.push({ text: todoInput.value });
      todoInput.value = '';
    } else {
      channel.updatePresence({ isTyping: true });
    }
  });

  todoInput.addEventListener('blur', () => {
    channel.updatePresence({ isTyping: false });
  });

  function render() {
    todosContainer.innerHTML = '';

    todos.forEach((todo, index) => {
      const todoContainer = document.createElement('div');
      todoContainer.classList.add('todo_container');

      const todoText = document.createElement('div');
      todoText.classList.add('todo');
      todoText.innerHTML = todo.text;
      todoContainer.appendChild(todoText);

      const deleteButton = document.createElement('button');
      deleteButton.classList.add('delete_button');
      deleteButton.innerHTML = '✕';
      deleteButton.addEventListener('click', () => {
        todos.delete(index, 1);
      });
      todoContainer.appendChild(deleteButton);

      todosContainer.appendChild(todoContainer);
    });
  }

  storage.subscribe(todos, () => {
    render();
  });

  render();
}

run();
