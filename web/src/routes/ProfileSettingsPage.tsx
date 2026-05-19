import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { FormEvent, useEffect, useState } from 'react';
import { Link } from 'react-router-dom';

import { useAuth } from '../features/auth/AuthProvider';
import { AvatarCropper } from '../features/profile/AvatarCropper';
import { updateProfile, requestAvatarUpload, uploadAvatarBlob, type Profile } from '../features/profile/api';
import { useSessionStore } from '../store/sessionStore';

export function ProfileSettingsPage() {
  const auth = useAuth();
  const queryClient = useQueryClient();
  const setSessionUser = useSessionStore((state) => state.setSessionUser);
  const [displayName, setDisplayName] = useState('');
  const [bio, setBio] = useState('');
  const [link, setLink] = useState('');
  const [isPrivate, setPrivate] = useState(false);
  const [avatarBlob, setAvatarBlob] = useState<Blob | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const profileQuery = useQuery({
    queryKey: ['me'],
    queryFn: () => updateProfile({}),
    enabled: auth.isAuthenticated,
  });

  useEffect(() => {
    if (!profileQuery.data) {
      return;
    }

    setDisplayName(profileQuery.data.display_name);
    setBio(profileQuery.data.bio);
    setLink(profileQuery.data.link ?? '');
    setPrivate(profileQuery.data.is_private);
    setSessionUser(sessionUserFromProfile(profileQuery.data));
  }, [profileQuery.data, setSessionUser]);

  const saveProfile = useMutation({
    mutationFn: () =>
      updateProfile({
        display_name: displayName,
        bio,
        link: link.trim() ? link : null,
        is_private: isPrivate,
      }),
    onSuccess: (profile) => {
      setMessage('Profile saved.');
      setError(null);
      setSessionUser(sessionUserFromProfile(profile));
      queryClient.setQueryData(['me'], profile);
      queryClient.setQueryData(['profile', profile.handle], profile);
    },
    onError: (requestError) => {
      setMessage(null);
      setError(errorMessage(requestError));
    },
  });

  const uploadAvatar = useMutation({
    mutationFn: async () => {
      if (!avatarBlob) {
        throw new Error('Choose an avatar image first.');
      }

      const upload = await requestAvatarUpload('image/jpeg');
      await uploadAvatarBlob(upload.upload_url, avatarBlob);
      return upload.user;
    },
    onSuccess: (profile) => {
      setMessage('Avatar saved.');
      setError(null);
      setSessionUser(sessionUserFromProfile(profile));
      queryClient.setQueryData(['me'], profile);
      queryClient.setQueryData(['profile', profile.handle], profile);
    },
    onError: (requestError) => {
      setMessage(null);
      setError(errorMessage(requestError));
    },
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setMessage(null);
    setError(null);
    saveProfile.mutate();
  }

  if (profileQuery.isLoading) {
    return (
      <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <p className="text-sm font-medium text-slate-600">Loading profile</p>
      </div>
    );
  }

  if (profileQuery.isError || !profileQuery.data) {
    return (
      <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <h1 className="text-2xl font-semibold">Profile settings unavailable</h1>
        <Link to="/login" className="mt-4 inline-block text-sm font-semibold text-emerald-700">
          Sign in
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <div className="flex items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-semibold">Profile settings</h1>
            <p className="mt-1 text-sm text-slate-600">@{profileQuery.data.handle}</p>
          </div>
          <Link to={`/u/${profileQuery.data.handle}`} className="text-sm font-semibold text-emerald-700">
            View profile
          </Link>
        </div>

        <form className="mt-6 grid gap-4" onSubmit={handleSubmit}>
          <label className="block">
            <span className="text-sm font-medium text-slate-700">Display name</span>
            <input
              value={displayName}
              onChange={(event) => setDisplayName(event.target.value)}
              className="mt-2 w-full rounded-md border border-slate-300 px-3 py-2 text-sm outline-none focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100"
              required
            />
          </label>

          <label className="block">
            <span className="text-sm font-medium text-slate-700">Bio</span>
            <textarea
              value={bio}
              onChange={(event) => setBio(event.target.value)}
              rows={4}
              className="mt-2 w-full resize-none rounded-md border border-slate-300 px-3 py-2 text-sm outline-none focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100"
            />
          </label>

          <label className="block">
            <span className="text-sm font-medium text-slate-700">Link</span>
            <input
              value={link}
              onChange={(event) => setLink(event.target.value)}
              className="mt-2 w-full rounded-md border border-slate-300 px-3 py-2 text-sm outline-none focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100"
              inputMode="url"
            />
          </label>

          <label className="flex items-center gap-3">
            <input
              type="checkbox"
              checked={isPrivate}
              onChange={(event) => setPrivate(event.target.checked)}
              className="size-4 rounded border-slate-300 text-slate-950 accent-slate-950"
            />
            <span className="text-sm font-medium text-slate-700">Private account</span>
          </label>

          {message ? <p className="rounded-md bg-emerald-50 px-3 py-2 text-sm text-emerald-800">{message}</p> : null}
          {error ? <p className="rounded-md bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}

          <button
            type="submit"
            disabled={saveProfile.isPending}
            className="w-full rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-400 sm:w-auto"
          >
            {saveProfile.isPending ? 'Saving' : 'Save profile'}
          </button>
        </form>
      </section>

      <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <h2 className="text-lg font-semibold">Avatar</h2>
        <div className="mt-4">
          <AvatarCropper onChange={setAvatarBlob} />
        </div>
        <button
          type="button"
          onClick={() => uploadAvatar.mutate()}
          disabled={!avatarBlob || uploadAvatar.isPending}
          className="mt-5 w-full rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-400 sm:w-auto"
        >
          {uploadAvatar.isPending ? 'Uploading' : 'Upload avatar'}
        </button>
      </section>
    </div>
  );
}

function sessionUserFromProfile(profile: Profile) {
  return {
    id: profile.id,
    email: profile.email ?? undefined,
    handle: profile.handle,
    display_name: profile.display_name,
    avatar_key: profile.avatar_key,
  };
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : 'Request failed';
}
