import { createBrowserRouter } from 'react-router-dom';

import { RootLayout } from './shell/RootLayout';
import { DirectMessagesPage } from '../routes/DirectMessagesPage';
import { ExplorePage } from '../routes/ExplorePage';
import { HomePage } from '../routes/HomePage';
import { LoginPage } from '../routes/LoginPage';
import { NotFoundPage } from '../routes/NotFoundPage';
import { PostPage } from '../routes/PostPage';
import { ProfilePage } from '../routes/ProfilePage';

export const router = createBrowserRouter([
  {
    path: '/',
    element: <RootLayout />,
    children: [
      { index: true, element: <HomePage /> },
      { path: 'login', element: <LoginPage /> },
      { path: 'explore', element: <ExplorePage /> },
      { path: 'p/:id', element: <PostPage /> },
      { path: 'u/:handle', element: <ProfilePage /> },
      { path: 'dm', element: <DirectMessagesPage /> },
      { path: '*', element: <NotFoundPage /> },
    ],
  },
]);
