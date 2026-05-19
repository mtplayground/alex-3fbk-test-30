import { expect, test, type Page, type Route } from '@playwright/test';

const now = new Date().toISOString();
const alice = {
  id: '00000000-0000-4000-8000-000000000001',
  email: 'alice@example.com',
  handle: 'alice',
  display_name: 'Alice',
  bio: '',
  link: null,
  avatar_key: null,
  is_private: false,
  email_verified: true,
};
const bob = {
  id: '00000000-0000-4000-8000-000000000002',
  handle: 'bob',
  display_name: 'Bob',
  bio: 'Story maker',
  link: null,
  avatar_key: null,
  is_private: false,
  email_verified: true,
};
const post = {
  id: '00000000-0000-4000-8000-000000000101',
  author: { id: alice.id, handle: alice.handle },
  caption: 'Hello #studio @bob',
  location: 'Test Studio',
  created_at: now,
  media: [
    {
      media_id: '00000000-0000-4000-8000-000000000201',
      position: 0,
      kind: 'image',
      original_key: 'media/originals/post.jpg',
      variants: {},
      width: null,
      height: null,
      duration_ms: null,
    },
  ],
  hashtags: ['studio'],
  mentions: [{ user_id: bob.id, handle: bob.handle, position: 15 }],
};
const story = {
  id: '00000000-0000-4000-8000-000000000301',
  author: bob,
  media: {
    media_id: '00000000-0000-4000-8000-000000000302',
    kind: 'image',
    status: 'ready',
    original_key: 'stories/bob.jpg',
    variants: {},
    width: null,
    height: null,
    duration_ms: null,
  },
  created_at: now,
  expires_at: new Date(Date.now() + 3_600_000).toISOString(),
  viewer_count: 0,
  viewed_at: null,
};
const conversation = {
  id: '00000000-0000-4000-8000-000000000401',
  kind: 'dm',
  title: null,
  created_at: now,
  updated_at: now,
  members: [
    { user_id: alice.id, joined_at: now, last_read_message_id: null },
    { user_id: bob.id, joined_at: now, last_read_message_id: null },
  ],
};

test('signup to social round trip', async ({ page }) => {
  const app = await installMockApi(page);

  await page.goto('/signup');
  await page.getByLabel('Email').fill(alice.email);
  await page.getByLabel('Handle').fill(alice.handle);
  await page.getByLabel('Display name').fill(alice.display_name);
  await page.getByLabel('Password').fill('correct horse battery staple');
  await page.getByRole('button', { name: 'Create account' }).click();

  await expect(page.getByRole('heading', { name: 'Home' })).toBeVisible();
  await page.locator('input[type="file"]').first().setInputFiles({
    name: 'post.gif',
    mimeType: 'image/gif',
    buffer: tinyGif(),
  });
  await expect(page.getByText('Complete')).toBeVisible();
  await page.getByLabel('Caption').fill(post.caption);
  await page.getByLabel('Location').fill(post.location);
  await page.getByRole('button', { name: 'Publish' }).click();
  await expect(page.getByText('Post published.')).toBeVisible();

  await page.goto(`/p/${post.id}`);
  await expect(page.getByText(post.caption)).toBeVisible();
  await page.getByRole('button', { name: 'Like' }).click();
  await expect(page.getByRole('button', { name: /Liked/ })).toBeVisible();
  await page.getByPlaceholder('Add a comment').fill('Great shot');
  await page.getByRole('button', { name: 'Post comment' }).click();
  await expect(page.getByText('Great shot')).toBeVisible();

  await page.goto('/u/bob');
  await expect(page.getByRole('heading', { name: 'Bob' })).toBeVisible();
  await page.getByRole('button', { name: 'Follow', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Following', exact: true })).toBeVisible();

  await page.goto(`/dm/${conversation.id}`);
  await page.getByPlaceholder('Write a message').fill('Round-trip DM');
  await page.getByRole('button', { name: 'Send' }).click();
  await expect(page.getByText('Round-trip DM')).toBeVisible();

  await page.goto('/');
  await page.getByRole('button', { name: /@bob/ }).click();
  await expect(page.getByText('@bob').first()).toBeVisible();
  await expect.poll(() => app.storyViewed).toBe(true);
  await page.getByRole('button', { name: 'Close' }).click();

  await page.goto('/notifications');
  await expect(page.getByText('3 unread')).toBeVisible();
  await expect(page.getByText('Commented on your post.')).toBeVisible();
  await expect(page.getByText('Started following you.')).toBeVisible();
  await expect(page.getByText('Sent you a message.')).toBeVisible();
});

async function installMockApi(page: Page) {
  const comments = [] as unknown[];
  const messages = [] as unknown[];
  const followers = [] as unknown[];
  const notifications = [
    notification('comment', 'post', post.id),
    notification('follow', 'user', alice.id),
    notification('dm', 'conversation', conversation.id),
  ];
  const state = { signedIn: false, storyViewed: false };

  await page.route('**/*', async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const path = url.pathname;
    const method = request.method();

    if (url.hostname === 'uploads.zeroclaw.test') {
      await route.fulfill({ status: 200, body: '' });
      return;
    }
    if (request.resourceType() === 'document') {
      await route.continue();
      return;
    }
    if (path === '/auth/refresh' && method === 'POST') {
      if (state.signedIn) {
        await route.fulfill({ json: { access_token: 'test-access-token', token_type: 'Bearer', expires_in: 900 } });
        return;
      }
      await route.fulfill({ status: 401, json: { error: { code: 'unauthorized' } } });
      return;
    }
    if (path === '/auth/signup' && method === 'POST') {
      state.signedIn = true;
      await route.fulfill({ json: authResponse() });
      return;
    }
    if (path === '/stories/feed' && method === 'GET') {
      await route.fulfill({ json: { authors: [{ author: bob, stories: [story] }] } });
      return;
    }
    if (path === `/stories/${story.id}/view` && method === 'POST') {
      state.storyViewed = true;
      await route.fulfill({ status: 204, body: '' });
      return;
    }
    if (path === '/feed' && method === 'GET') {
      await route.fulfill({ json: { posts: [], next_cursor: null } });
      return;
    }
    if (path === '/media/uploads' && method === 'POST') {
      await route.fulfill({
        json: {
          asset_id: post.media[0].media_id,
          key: 'media/originals/post.jpg',
          upload_url: 'https://uploads.zeroclaw.test/post.jpg',
          method: 'PUT',
          expires_in: 900,
        },
      });
      return;
    }
    if (path === `/media/uploads/${post.media[0].media_id}/complete` && method === 'POST') {
      await route.fulfill({
        json: {
          asset_id: post.media[0].media_id,
          status: 'uploaded',
          job_id: '00000000-0000-4000-8000-000000000202',
          job_kind: 'image_processing',
        },
      });
      return;
    }
    if (path === '/posts' && method === 'POST') {
      await route.fulfill({ json: post });
      return;
    }
    if (path === `/posts/${post.id}` && method === 'GET') {
      await route.fulfill({ json: post });
      return;
    }
    if (path === `/posts/${post.id}/like` && method === 'POST') {
      await route.fulfill({ json: { active: true, count: 1 } });
      return;
    }
    if (path === `/posts/${post.id}/comments` && method === 'GET') {
      await route.fulfill({ json: { comments } });
      return;
    }
    if (path === `/posts/${post.id}/comments` && method === 'POST') {
      const comment = {
        id: '00000000-0000-4000-8000-000000000501',
        post_id: post.id,
        parent_id: null,
        author: { id: alice.id, handle: alice.handle },
        body: 'Great shot',
        created_at: now,
        replies: [],
      };
      comments.push(comment);
      await route.fulfill({ json: comment });
      return;
    }
    if (path === '/users/bob' && method === 'GET') {
      await route.fulfill({ json: bob });
      return;
    }
    if (path === '/users/bob/posts' && method === 'GET') {
      await route.fulfill({ json: { posts: [], next_cursor: null } });
      return;
    }
    if (path === '/users/bob/followers' && method === 'GET') {
      await route.fulfill({ json: { users: followers } });
      return;
    }
    if (path === '/users/bob/following' && method === 'GET') {
      await route.fulfill({ json: { users: [] } });
      return;
    }
    if (path === '/users/bob/follow' && method === 'POST') {
      followers.push({
        id: alice.id,
        handle: alice.handle,
        display_name: alice.display_name,
        avatar_key: null,
        is_private: false,
      });
      await route.fulfill({ json: { follower_id: alice.id, followee_id: bob.id, state: 'accepted' } });
      return;
    }
    if (path === '/conversations' && method === 'GET') {
      await route.fulfill({ json: { conversations: [conversation] } });
      return;
    }
    if (path === `/conversations/${conversation.id}/messages` && method === 'GET') {
      await route.fulfill({ json: { messages, next_cursor: null } });
      return;
    }
    if (path === `/conversations/${conversation.id}/messages` && method === 'POST') {
      const message = {
        id: '00000000-0000-4000-8000-000000000601',
        conversation_id: conversation.id,
        author_id: alice.id,
        body: 'Round-trip DM',
        media_id: null,
        created_at: now,
      };
      messages.push(message);
      await route.fulfill({ json: message });
      return;
    }
    if (path === `/conversations/${conversation.id}/read` && method === 'POST') {
      await route.fulfill({
        json: { user_id: alice.id, joined_at: now, last_read_message_id: messages.at(-1)?.id ?? null },
      });
      return;
    }
    if (path === '/notifications' && method === 'GET') {
      await route.fulfill({ json: { notifications, next_cursor: null } });
      return;
    }
    if (path === '/notifications/unread-count' && method === 'GET') {
      await route.fulfill({ json: { unread_count: notifications.length } });
      return;
    }

    await route.continue();
  });

  return state;
}

function authResponse() {
  return {
    access_token: 'test-access-token',
    token_type: 'Bearer',
    expires_in: 900,
    user: alice,
  };
}

function notification(kind: string, target_kind: string, target_id: string) {
  return {
    id: crypto.randomUUID(),
    user_id: alice.id,
    kind,
    actor_id: bob.id,
    target_kind,
    target_id,
    read_at: null,
    created_at: now,
  };
}

function tinyGif() {
  return Buffer.from(
    'R0lGODlhAQABAPAAAP///wAAACH5BAAAAAAALAAAAAABAAEAAAICRAEAOw==',
    'base64',
  );
}
