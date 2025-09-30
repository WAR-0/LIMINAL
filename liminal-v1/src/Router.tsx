import { useState, useEffect } from 'react';
import App from './App';
import { ComponentShowcase } from './playground/ComponentShowcase';

export function Router() {
  const [route, setRoute] = useState(window.location.hash.slice(1) || '/');

  useEffect(() => {
    const handleHashChange = () => {
      setRoute(window.location.hash.slice(1) || '/');
    };

    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  if (route === '/playground') {
    return <ComponentShowcase />;
  }

  return <App />;
}