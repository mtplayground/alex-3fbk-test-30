const posts = [
  {
    id: 'sunrise-atelier',
    author: 'mira',
    accent: 'from-rose-300 to-amber-300',
    caption: 'Morning edit stack with warm contrast and clean shadows.',
  },
  {
    id: 'city-lines',
    author: 'atlas',
    accent: 'from-cyan-300 to-blue-500',
    caption: 'Glass, concrete, and the long walk between platforms.',
  },
  {
    id: 'green-room',
    author: 'noor',
    accent: 'from-emerald-300 to-lime-500',
    caption: 'Set notes, product swatches, and one very precise color pass.',
  },
];

export function HomePage() {
  return (
    <div className="space-y-5">
      <div className="flex items-end justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold">Home</h1>
          <p className="mt-1 text-sm text-slate-600">Latest posts from followed accounts.</p>
        </div>
      </div>

      <div className="grid gap-5">
        {posts.map((post) => (
          <article key={post.id} className="overflow-hidden rounded-lg border border-slate-200 bg-white shadow-soft">
            <div className={`aspect-[16/9] bg-gradient-to-br ${post.accent}`} />
            <div className="space-y-3 p-4">
              <div className="flex items-center justify-between">
                <a href={`/u/${post.author}`} className="font-semibold text-slate-950">
                  @{post.author}
                </a>
                <a href={`/p/${post.id}`} className="text-sm font-medium text-emerald-700">
                  Open
                </a>
              </div>
              <p className="text-sm leading-6 text-slate-700">{post.caption}</p>
            </div>
          </article>
        ))}
      </div>
    </div>
  );
}
