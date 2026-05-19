const tiles = [
  'bg-fuchsia-300',
  'bg-emerald-300',
  'bg-sky-300',
  'bg-orange-300',
  'bg-violet-300',
  'bg-lime-300',
  'bg-rose-300',
  'bg-teal-300',
  'bg-amber-300',
];

export function ExplorePage() {
  return (
    <div className="space-y-5">
      <h1 className="text-2xl font-semibold">Explore</h1>
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
        {tiles.map((tile, index) => (
          <a
            key={`${tile}-${index}`}
            href={`/p/explore-${index + 1}`}
            className={`aspect-square rounded-lg ${tile} transition hover:scale-[1.01]`}
            aria-label={`Explore post ${index + 1}`}
          />
        ))}
      </div>
    </div>
  );
}
