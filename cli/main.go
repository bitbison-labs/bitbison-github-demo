// corral: the link shortener's command-line client.
//
//	corral add https://example.com/a/long/address
//	corral ls
//
// It talks to the corral API at --api, or CORRAL_API, or
// http://127.0.0.1:8080.
package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

func main() {
	api := os.Getenv("CORRAL_API")
	if api == "" {
		api = "http://127.0.0.1:8080"
	}

	root := &cobra.Command{
		Use:           "corral",
		Short:         "Short links, and how often each is followed",
		SilenceUsage:  true,
		SilenceErrors: true,
	}
	root.PersistentFlags().StringVar(&api, "api", api, "the corral API's address")

	root.AddCommand(&cobra.Command{
		Use:   "add <url>",
		Short: "Shorten a URL",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			link, err := (&Client{Base: api}).Add(args[0])
			if err != nil {
				return err
			}
			fmt.Fprintf(cmd.OutOrStdout(), "%s/%s\n", api, link.Slug)
			return nil
		},
	})

	root.AddCommand(&cobra.Command{
		Use:   "ls",
		Short: "List every link, most followed first",
		Args:  cobra.NoArgs,
		RunE: func(cmd *cobra.Command, args []string) error {
			links, err := (&Client{Base: api}).Links()
			if err != nil {
				return err
			}
			fmt.Fprint(cmd.OutOrStdout(), Table(links))
			return nil
		},
	})

	if err := root.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, "corral:", err)
		os.Exit(1)
	}
}
