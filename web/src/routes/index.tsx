import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")({
	component: Home,
});

function Home() {
	return (
		<main className="page-wrap px-4 py-16 flex flex-col items-center justify-center text-center">
			<h1 className="text-4xl font-extrabold tracking-tight text-[var(--sea-ink)] mb-4">
				Hello from homepage
			</h1>
		</main>
	);
}
