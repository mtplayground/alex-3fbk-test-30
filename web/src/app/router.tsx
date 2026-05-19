import { createBrowserRouter } from 'react-router-dom';

import { RootLayout } from './shell/RootLayout';
import { ProtectedRoute } from '../features/auth/ProtectedRoute';
import { DirectMessagesPage } from '../routes/DirectMessagesPage';
import { ExplorePage } from '../routes/ExplorePage';
import { ForgotPasswordPage } from '../routes/ForgotPasswordPage';
import { HomePage } from '../routes/HomePage';
import { LoginPage } from '../routes/LoginPage';
import { NotFoundPage } from '../routes/NotFoundPage';
import { PostPage } from '../routes/PostPage';
import { ProfilePage } from '../routes/ProfilePage';
import { ProfileSettingsPage } from '../routes/ProfileSettingsPage';
import { ReelsPage } from '../routes/ReelsPage';
import { ResetPasswordPage } from '../routes/ResetPasswordPage';
import { SignupPage } from '../routes/SignupPage';

export const router = createBrowserRouter([
  {
    element: <ProtectedRoute />,
    children: [{ path: '/reels', element: <ReelsPage /> }],
  },
  {
    path: '/',
    element: <RootLayout />,
    children: [
      { index: true, element: <HomePage /> },
      { path: 'login', element: <LoginPage /> },
      { path: 'signup', element: <SignupPage /> },
      { path: 'forgot-password', element: <ForgotPasswordPage /> },
      { path: 'reset-password', element: <ResetPasswordPage /> },
      { path: 'explore', element: <ExplorePage /> },
      { path: 'p/:id', element: <PostPage /> },
      { path: 'u/:handle', element: <ProfilePage /> },
      {
        element: <ProtectedRoute />,
        children: [
          { path: 'dm', element: <DirectMessagesPage /> },
          { path: 'dm/:conversationId', element: <DirectMessagesPage /> },
          { path: 'settings/profile', element: <ProfileSettingsPage /> },
        ],
      },
      { path: '*', element: <NotFoundPage /> },
    ],
  },
]);
