package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"text/tabwriter"
	"time"
)

// Link is a short link as the API gives it.
type Link struct {
	Slug string `json:"slug"`
	URL  string `json:"url"`
	Hits uint64 `json:"hits"`
}

// Client talks to the corral API at Base.
type Client struct {
	Base string
	HTTP *http.Client
}

func (c *Client) http() *http.Client {
	if c.HTTP != nil {
		return c.HTTP
	}
	return &http.Client{Timeout: 10 * time.Second}
}

// decode reads the API's answer into out, or its error.
func decode(response *http.Response, out any) error {
	defer response.Body.Close()
	if response.StatusCode >= 300 {
		var problem struct {
			Error string `json:"error"`
		}
		if json.NewDecoder(response.Body).Decode(&problem) == nil && problem.Error != "" {
			return fmt.Errorf("%s", problem.Error)
		}
		return fmt.Errorf("the API answered %s", response.Status)
	}
	return json.NewDecoder(response.Body).Decode(out)
}

// Links lists every link, most followed first.
func (c *Client) Links() ([]Link, error) {
	response, err := c.http().Get(strings.TrimRight(c.Base, "/") + "/links")
	if err != nil {
		return nil, err
	}
	var links []Link
	return links, decode(response, &links)
}

// Add shortens a URL.
func (c *Client) Add(url string) (Link, error) {
	body, err := json.Marshal(map[string]string{"url": url})
	if err != nil {
		return Link{}, err
	}
	response, err := c.http().Post(
		strings.TrimRight(c.Base, "/")+"/links", "application/json", bytes.NewReader(body))
	if err != nil {
		return Link{}, err
	}
	var link Link
	return link, decode(response, &link)
}

// Table renders links in aligned columns.
func Table(links []Link) string {
	var out bytes.Buffer
	w := tabwriter.NewWriter(&out, 0, 0, 2, ' ', 0)
	fmt.Fprintln(w, "LINK\tHITS\tGOES TO")
	for _, l := range links {
		fmt.Fprintf(w, "%s\t%d\t%s\n", l.Slug, l.Hits, l.URL)
	}
	w.Flush()
	return out.String()
}
