const threads = ['mira', 'atlas', 'noor'];

export function DirectMessagesPage() {
  return (
    <div className="overflow-hidden rounded-lg border border-slate-200 bg-white shadow-soft">
      <div className="border-b border-slate-200 p-5">
        <h1 className="text-2xl font-semibold">Messages</h1>
      </div>
      <div className="divide-y divide-slate-200">
        {threads.map((thread) => (
          <button key={thread} className="flex w-full items-center gap-3 px-5 py-4 text-left hover:bg-slate-50">
            <span className="grid size-10 place-items-center rounded-full bg-cyan-100 font-semibold text-cyan-950">
              {thread.slice(0, 1).toUpperCase()}
            </span>
            <span>
              <span className="block text-sm font-semibold">@{thread}</span>
              <span className="block text-sm text-slate-600">Thread preview</span>
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
